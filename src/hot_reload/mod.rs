//! Hot reload
use core::{
    hash::{Hash, Hasher},
    iter,
};
use std::{env, path::Path};

use bevy::{
    asset::{
        io::{AssetSourceBuilder, Reader},
        AssetLoader, AsyncReadExt, LoadContext,
    },
    ecs::system::SystemState,
    prelude::*,
    reflect::{
        DynamicEnum, DynamicStruct, DynamicTuple, DynamicTupleStruct, DynamicVariant, ReflectKind,
        TypeInfo, TypeRegistry,
    },
    utils::{AHasher, HashMap, HashSet, Hashed},
};
use cant_wait_for_bsn_parse::*;
use syn::{spanned::Spanned, visit::Visit, Expr, FieldValue, Member};
use thiserror::Error;
use visit::BsnMacroVisitor;

use crate::{
    ConstructContext, ConstructError, DynamicScene, ReflectConstruct, ReflectFromBsn, Scene,
};

/// Extension trait for [`App`] to add hot-reload sources for BSN macros.
pub trait BsnHotReloadAppExt {
    /// Registers a source directory for hot-reloading BSN macros.
    fn register_bsn_hot_reload_source(&mut self, dir: &'static str) -> &mut Self;
}

impl BsnHotReloadAppExt for App {
    fn register_bsn_hot_reload_source(&mut self, dir: &'static str) -> &mut Self {
        self.register_asset_source(
            dir,
            AssetSourceBuilder::platform_default(
                format!(
                    "{}/{}",
                    &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| "".to_string()),
                    dir,
                )
                .as_str(),
                None,
            ),
        )
    }
}

/// Adds hot-reload support for BSN macros.
pub struct BsnHotReloadPlugin;

impl Plugin for BsnHotReloadPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<BsnRustFile>();
        app.init_asset_loader::<HotBsnMacroLoader>();
        app.init_resource::<HotReloadState>();
        app.add_systems(PreStartup, initialize_hot_reload);
        app.add_systems(Update, hot_reload_bsn);
    }
}

fn initialize_hot_reload(mut state: ResMut<HotReloadState>, asset_server: Res<AssetServer>) {
    // TODO: Actually use the non-hardcoded, configured sources
    // let handle = asset_server.load::<BsnRustFile>(format!(
    //     "examples://",
    //     Path::new(file!()).file_name().unwrap().to_string_lossy()
    // ));
    let handle = asset_server.load::<BsnRustFile>("examples://hot_reload.rs");
    state.handles.insert(handle.id(), handle);
}

/// State resource for hot-reloading BSN macros.
#[derive(Resource, Default)]
pub struct HotReloadState {
    /// Handles to the source files.
    pub handles: HashMap<AssetId<BsnRustFile>, Handle<BsnRustFile>>,
    /// Map from asset id to the invocation ids of the BSN macros in the source file.
    pub invocation_ids: HashMap<AssetId<BsnRustFile>, Vec<BsnInvocationId>>,
    /// Map from invocation id to the dynamic scene constructed from the BSN macro.
    pub hot_scenes: HashMap<BsnInvocationId, DynamicScene>,
}

/// Identifies a specific bsn! macro invocation in the _original_ source files.
#[derive(Debug, Hash, PartialEq, Eq, Deref, DerefMut, Copy, Clone, Reflect)]
pub struct BsnInvocationId(u64);

impl BsnInvocationId {
    /// Creates a new [`BsnInvocationId`] by hashing the given `path`, `line`, and `column`.
    pub fn new(path: &str, line: u32, column: u32) -> Self {
        let mut hasher = AHasher::default();
        path.hash(&mut hasher);
        line.hash(&mut hasher);
        column.hash(&mut hasher);
        Self(hasher.finish())
    }
}

/// An invocation of the bsn! macro
#[derive(TypePath, Debug)]
pub struct BsnMacroInvocation {
    /// Line number (1-based) of this macro invocation.
    pub line: usize,
    /// Column number (1-based) of this macro invocation.
    pub column: usize,
    /// The parsed BSN ast.
    pub hashed_bsn: Hashed<BsnEntity>,
    /// Maps idents to paths of named `use` declarations that are in scope for this invocation.
    pub named_uses: HashMap<String, String>,
    /// The paths of the glob `use` declarations that are in scope for this invocation. Excluding the `::*`,
    pub glob_uses: Vec<String>,
}

#[allow(unsafe_code)]
// SAFETY: Todo (:
unsafe impl Send for BsnMacroInvocation {}
#[allow(unsafe_code)]
// SAFETY: Todo (:
unsafe impl Sync for BsnMacroInvocation {}

/// A rust source file loaded as an asset.
#[derive(Asset, TypePath, Debug)]
pub struct BsnRustFile {
    /// Path
    pub path: String,
    /// Contents of the source file
    pub content: String,
    // /// List of BSN macro invocations in this file.
    // pub invocations: Vec<BsnMacroInvocation>,
}

/// Asset loader for hot reloading BSN
#[derive(Default)]
pub struct HotBsnMacroLoader;

/// Error for [`HotBsnMacroLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum HotBsnMacroLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load source file: {0}")]
    Io(#[from] std::io::Error),
    /// A [Syn](syn) Error
    #[error("Failed to parse source file: {0}")]
    Syn(#[from] syn::Error),
}

impl AssetLoader for HotBsnMacroLoader {
    type Asset = BsnRustFile;
    type Settings = ();
    type Error = HotBsnMacroLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut content = String::new();
        reader.read_to_string(&mut content).await?;

        // let path = world.resource_scope(|_, asset_server: Mut<AssetServer>| {
        //     let asset_path = asset_server.get_path(*id).unwrap();
        //     Path::join(
        //         Path::new(asset_path.source().as_str().unwrap_or("")),
        //         asset_path.path(),
        //     )
        //     .to_string_lossy()
        //     .to_string()
        // });
        let asset_path = load_context.asset_path();
        let path = Path::join(
            Path::new(asset_path.source().as_str().unwrap_or("")),
            asset_path.path(),
        )
        .to_string_lossy()
        .to_string();

        // let ast = syn::parse_file(&content)?;

        // let mut visitor = BsnMacroVisitor::default();
        // visitor.visit_file(&ast);

        // let invocations = visitor
        //     .invocations
        //     .into_iter()
        //     .map(|(uses, invocation)| {
        //         info!("Uses: {:?}", uses);
        //         let bsn = syn::parse2::<BsnEntity>(invocation.tokens.clone())?;
        //         let span = invocation.span();
        //         Ok(BsnMacroInvocation {
        //             line: span.start().line,
        //             column: span.start().column + 1, // TODO: UTF-8 and stuff
        //             hashed_bsn: Hashed::new(bsn),
        //             uses,
        //         })
        //     })
        //     .collect::<syn::Result<Vec<_>>>()?;

        //Ok(BsnRustFile { ast, invocations })
        Ok(BsnRustFile { path, content })
    }

    fn extensions(&self) -> &[&str] {
        &["rs"]
    }
}

/// Component holding the invocation ids for the hot-reloadable scenes that have been constructed on this entity.
#[derive(Default, Component, Deref, DerefMut, Reflect)]
pub struct HotReloadScenes(pub HashSet<BsnInvocationId>);

/// A hot-reloadable scene originating from a bsn! macro invocation.
pub struct HotReloadableBsnMacro<T: Scene> {
    /// Source file path of this macro invocation.
    pub file: &'static str,
    /// Line number (1-based) of this macro invocation.
    pub line: u32,
    /// Column number (1-based) of this macro invocation.
    pub column: u32,
    /// ID of this macro invocation.
    pub id: BsnInvocationId,
    /// Scene
    pub scene: T,
}

impl<T: Scene> Scene for HotReloadableBsnMacro<T> {
    fn construct(self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        info!(
            "CONSTRUCTING: file: {}, line: {}, column: {}, id: {:?}",
            self.file, self.line, self.column, self.id
        );

        {
            // Add the id to the entity's tracked hot-reloadable scenes
            let mut entity = context.world.entity_mut(context.id);
            let mut hot_scenes = entity.entry::<HotReloadScenes>().or_default();
            hot_scenes.insert(self.id);
        }

        // TODO: Use the id to look up if we should use a hot-reloaded scene or the original one
        // Use original for now:
        self.scene.construct(context)?;

        Ok(())
    }

    fn spawn(self, context: &mut ConstructContext) -> Result<(), ConstructError> {
        warn!("TODO: Hot-reloading scenes are not supported for spawning yet.");
        self.scene.spawn(context)
    }

    fn dynamic_patch(&mut self, scene: &mut DynamicScene) {
        warn!("TODO: Hot-reloading scenes are not supported for dynamic_patch yet.");
        self.scene.dynamic_patch(scene);
    }

    fn dynamic_patch_as_child(&mut self, scene: &mut DynamicScene) {
        warn!("TODO: Hot-reloading scenes are not supported for dynamic_patch_as_child yet.");
        self.scene.dynamic_patch_as_child(scene);
    }
}

fn hot_reload_bsn(
    world: &mut World,
    event_reader: &mut SystemState<EventReader<AssetEvent<BsnRustFile>>>,
) {
    // TODO: Clean up this mess
    world.resource_scope(|world, mut state: Mut<HotReloadState>| {
        world.resource_scope(|world, assets: Mut<Assets<BsnRustFile>>| {
            let events = {
                let mut event_reader = event_reader.get_mut(world);
                event_reader.read().cloned().collect::<Vec<_>>()
            };
            for ev in events.iter() {
                match ev {
                    AssetEvent::Added { id } => {
                        // TODO: Deal with the fact that source files might have changed between compiling and reaching this point.

                        let file = assets.get(*id).unwrap();
                        info!("Asset Created: {:?}", file.path);

                        // Parse file and visit bsn! invocations
                        let ast = match syn::parse_file(&file.content) {
                            Ok(ast) => ast,
                            Err(e) => {
                                error!("Failed to parse source file {}: {}", file.path, e);
                                continue;
                            }
                        };
                        let mut visitor = BsnMacroVisitor::default();
                        visitor.visit_file(&ast);

                        if visitor.invocations.is_empty() {
                            state.handles.remove(id);
                            continue;
                        }

                        // Store the invocation ids for this file
                        // TODO: Pre-hash things to allow detection of changed parts
                        let invocation_ids = visitor
                            .invocations
                            .iter()
                            .map(|invocation| {
                                let span = invocation.span();
                                BsnInvocationId::new(
                                    &file.path,
                                    span.start().line as u32,
                                    (span.start().column + 1) as u32, // TODO: UTF-8 and stuff
                                )
                            })
                            .collect();

                        state.invocation_ids.insert(*id, invocation_ids);
                    }
                    AssetEvent::Modified { id } => {
                        info!("Asset Modified: {:?}", id);
                        let file = assets.get(*id).unwrap();
                        let HotReloadState {
                            invocation_ids,
                            ..
                        } = state.as_mut();

                        // Parse file and visit bsn! invocations
                        let ast = match syn::parse_file(&file.content) {
                            Ok(ast) => ast,
                            Err(e) => {
                                error!("Failed to parse source file {}: {}", file.path, e);
                                continue;
                            }
                        };
                        let mut visitor = BsnMacroVisitor::default();
                        visitor.visit_file(&ast);
                        let invocations = visitor.invocations;
                        let invocation_ids = invocation_ids.get(id).unwrap();

                        // Ensure that the number of invocations has not changed because we (sadly) rely on index for identification between loads.
                        // TODO: Figure out if there is a better way to do this.
                        if invocations.len() != invocation_ids.len() {
                            warn!("bsn!-invocation count changed in {:?}, this file will not be hot reloaded until the next recompile.", file.path);
                            state.handles.remove(id);
                            continue;
                        }

                        for (invocation, invocation_id) in invocations
                            .into_iter()
                            .zip(invocation_ids.iter())
                        {
                            // TODO: Hashing to see if the scene needs to be reloaded.
                            // TODO2: Hashing for each patch to see if it needs to be reapplied.

                            // TODO: Component removal detection
                            // TODO: Children
                            // TODO: Inheritance (tricky tricky)

                            let bsn = match syn::parse2::<BsnEntity>(invocation.tokens.clone()) {
                                Ok(bsn) => bsn,
                                Err(e) => {
                                    error!(
                                        "Failed to parse bsn! invocation in {:?}: {}",
                                        file.path, e
                                    );
                                    continue;
                                }
                            };

                            // Find any entities currently using this invocation
                            let entities = world.query::<(Entity, &HotReloadScenes)>()
                                .iter(world)
                                .filter_map(|(entity, hot_scenes)| {
                                    if hot_scenes.contains(invocation_id) {
                                        return Some(entity);
                                    }
                                    None
                                })
                                .collect::<Vec<_>>();

                            // Hot-Patch those entities
                            for entity in entities {
                                // TODO: Not really necessary to do this dynamicscene tango for _each_ entity? Should DynamicScene::construct take &self instead?
                                let mut dynamic_scene = DynamicScene::default();
                                {
                                    let app_type_registry = world.resource::<AppTypeRegistry>();
                                    let registry = app_type_registry.read();
                                    add_parsed_patch_to_dynamic_scene(&mut dynamic_scene, &bsn.patch, &registry);
                                }

                                if let Err(e) = dynamic_scene.construct(&mut ConstructContext {
                                    world,
                                    id: entity,
                                }) {
                                    error!("Failed to hot-patch entity: {:?}", e);
                                } else {
                                    info!("Hot-patched entity: {:?}", entity);
                                }
                            }

                            // Store it for future construction of the patch.
                            // TODO: Only store incremental changes?
                            //   That way the original static patch can still be used as the base,
                            //   allowig non-reflectable/registered stuff to keep working if not touched between hot reloads.
                            //hot_scenes.insert(*invocation_id, dynamic_scene);
                        }
                    }
                    _ => (),
                }
            }
        });
    });
}

fn add_parsed_patch_to_dynamic_scene(
    dynamic_scene: &mut DynamicScene,
    patch: &BsnPatch,
    registry: &TypeRegistry,
) {
    // TODO: Resolve paths using reflection and parsed use declarations
    match patch {
        BsnPatch::Tuple(tuple) => {
            for patch in tuple {
                add_parsed_patch_to_dynamic_scene(dynamic_scene, patch, registry);
            }
        }
        BsnPatch::Patch(path, props) => {
            // TODO: Better path build (generics etc)
            let path = iter::once("".to_string())
                .filter(|_| path.leading_colon.is_some())
                .chain(path.segments.iter().map(|seg| seg.ident.to_string()))
                .collect::<Vec<String>>()
                .join("::");

            // TODO: Better path resolution. Could parse the use declarations and avoid ambiguities.
            let Some(component_type) = registry.get_with_short_type_path(&path) else {
                warn!("Failed to resolve component path `{:?}` to registered type. Hot-reload currently supports only unambiguous short paths.", path);
                return;
            };

            if props.is_empty() {
                dynamic_scene
                    .component_props
                    .insert(component_type.type_id(), Vec::new());
                return;
            }

            let Some(reflect_construct) = component_type.data::<ReflectConstruct>() else {
                warn!(
                    "No registered ReflectConstruct for component `{:?}`. Skipping hot-reload for this component. Consider adding #[reflect(Construct)].",
                    path
                );
                return;
            };

            let props_type = if reflect_construct.props_type_id == component_type.type_id() {
                component_type
            } else {
                let Some(props_type) = registry.get(reflect_construct.props_type_id) else {
                    warn!("Props with TypeId `{:?}` for component `{:?}` is not registered in the type regstry. Skipping hot-reload for this component.", reflect_construct.props_type_id, path);
                    return;
                };
                props_type
            };

            // TODO: Ugly ugly
            // TODO: Support construct props properly
            if props_type.type_info().kind() == ReflectKind::Struct {
                let props_struct = props_type.type_info().as_struct().unwrap();
                let mut dynamic_props = DynamicStruct::default();

                for (member, val) in props.iter() {
                    let Member::Named(name) = member else {
                        warn!("Got tuple struct for `{}` which is supposed to be a struct with named fields. Skipping hot-reload for this component.", props_type.type_info().type_path());
                        return;
                    };
                    let name = name.to_string();

                    let Some(field) = props_struct.field(&name) else {
                        warn!("Failed to resolve field `{}` in `{}`. Skipping hot-reload for this field.", name, props_type.type_info().type_path());
                        continue;
                    };

                    let val = match reflect_from_bsn_expr(
                        val.into(),
                        field.type_info().unwrap(),
                        registry,
                    ) {
                        Ok(val) => val,
                        Err(e) => {
                            warn!("Failed to reflect field `{}` in `{}`: {}. Skipping hot-reload for this field.", name, props_type.type_info().type_path(), e);
                            continue;
                        }
                    };

                    dynamic_props.insert_boxed(name, val.into_partial_reflect());
                }

                dynamic_scene.component_props.insert(
                    component_type.type_id(),
                    vec![Box::new(move |patch_props: &mut dyn Reflect| {
                        patch_props.apply(&dynamic_props);
                    })],
                );
            } else {
                let props_struct = props_type.type_info().as_tuple_struct().unwrap();
                let mut dynamic_props = DynamicTupleStruct::default();

                for (member, val) in props.iter() {
                    let Member::Unnamed(index) = member else {
                        warn!("Got struct with named fields for `{}` which is supposed to be a tuple struct. Skipping hot-reload for this component.", props_type.type_info().type_path());
                        return;
                    };
                    let index = index.index as usize;

                    let Some(field) = props_struct.field_at(index) else {
                        warn!("Failed to resolve field `{}` in `{}`. Skipping hot-reload for this field.", index, props_type.type_info().type_path());
                        continue;
                    };

                    let val = match reflect_from_bsn_expr(
                        val.into(),
                        field.type_info().unwrap(),
                        registry,
                    ) {
                        Ok(val) => val,
                        Err(e) => {
                            warn!("Failed to reflect field `{}` in `{}`: {}. Skipping hot-reload for this field.", index, props_type.type_info().type_path(), e);
                            continue;
                        }
                    };

                    dynamic_props.insert_boxed(val.into_partial_reflect());
                }

                dynamic_scene.component_props.insert(
                    component_type.type_id(),
                    vec![Box::new(move |patch_props: &mut dyn Reflect| {
                        patch_props.apply(&dynamic_props);
                    })],
                );
            };
        }
        BsnPatch::Expr(e) => {
            warn!(
                "Can't hot reload expression: `{:?}`. Skipping hot-reload for this component.",
                e
            );
        }
    }
}

fn reflect_from_bsn_expr(
    expr: &Expr,
    ty: &TypeInfo,
    registry: &TypeRegistry,
) -> Result<Box<dyn PartialReflect>, FromBsnError> {
    let reflect_from_bsn = registry.get_type_data::<ReflectFromBsn>(ty.type_id());
    let kind = ty.kind();

    // TODO: Try each one in order instead of matching one?
    // TODO: What about .into()? The reflect kind of T might be different from the type V of val if V implements Into<T>
    let val = match expr {
        _ if reflect_from_bsn.is_some() => match reflect_from_bsn.unwrap().from_bsn(expr.clone()) {
            Ok(val) => val.into_partial_reflect(),
            Err(e) => {
                return Err(e);
            }
        },
        Expr::Struct(expr) if kind == ReflectKind::Struct => {
            // Struct
            let struct_info = ty.as_struct().unwrap();
            let mut dynamic_struct = DynamicStruct::default();

            for FieldValue {
                member, expr: val, ..
            } in expr.fields.iter()
            {
                let Member::Named(name) = member else {
                    unreachable!()
                };
                let name = name.to_string();

                let Some(field) = struct_info.field(&name) else {
                    warn!(
                        "Failed to resolve field `{}` in `{}`. Skipping field.",
                        name,
                        ty.type_path()
                    );
                    continue;
                };

                let val = match reflect_from_bsn_expr(val, field.type_info().unwrap(), registry) {
                    Ok(val) => val,
                    Err(e) => {
                        warn!(
                            "Failed to reflect field `{}` in `{}`: {}. Skipping field.",
                            name,
                            ty.type_path(),
                            e
                        );
                        continue;
                    }
                };

                dynamic_struct.insert_boxed(name, val.into_partial_reflect());
            }

            Box::new(dynamic_struct)
        }
        Expr::Call(expr) if kind == ReflectKind::TupleStruct => {
            // Tuple struct
            let props_struct = ty.as_tuple_struct().unwrap();
            let mut dynamic_struct = DynamicTupleStruct::default();

            for (index, val) in expr.args.iter().enumerate() {
                let Some(field) = props_struct.field_at(index) else {
                    warn!(
                        "Failed to resolve field `{}` in `{}`. Skipping field.",
                        index,
                        ty.type_path()
                    );
                    continue;
                };

                let val = match reflect_from_bsn_expr(val, field.type_info().unwrap(), registry) {
                    Ok(val) => val,
                    Err(e) => {
                        warn!(
                            "Failed to reflect field `{}` in `{}`: {}. Skipping field.",
                            index,
                            ty.type_path(),
                            e
                        );
                        continue;
                    }
                };

                dynamic_struct.insert_boxed(val.into_partial_reflect());
            }
            Box::new(dynamic_struct)
        }
        Expr::Path(expr) if kind == ReflectKind::Enum => {
            // Enum (unit-like)
            let variant_name = expr.path.segments.last().unwrap().ident.to_string();
            let reflect_enum = ty.as_enum().unwrap();
            if !reflect_enum.contains_variant(&variant_name) {
                return Err(FromBsnError::Custom(
                    format!(
                        "Can't find enum variant `{}` for type `{}`",
                        variant_name,
                        ty.type_path()
                    )
                    .into(),
                ));
            }
            Box::new(DynamicEnum::new(&variant_name, DynamicVariant::Unit))
        }
        Expr::Call(expr) if kind == ReflectKind::Enum => {
            // Enum (tuple-like)
            let reflect_enum = ty.as_enum().unwrap();
            let variant_name = match expr.func.as_ref() {
                Expr::Path(expr) => expr.path.segments.last().unwrap().ident.to_string(),
                _ => {
                    return Err(FromBsnError::Custom(
                        format!(
                            "Failed to resolve enum variant path for type `{}`",
                            ty.type_path()
                        )
                        .into(),
                    ));
                }
            };

            let Some(variant) = reflect_enum.variant(&variant_name) else {
                return Err(FromBsnError::Custom(
                    format!(
                        "Can't find enum variant `{}` for type `{}`",
                        variant_name,
                        ty.type_path()
                    )
                    .into(),
                ));
            };
            let variant = variant.as_tuple_variant().unwrap();

            let mut dynamic_tuple = DynamicTuple::default();
            for (i, arg) in expr.args.iter().enumerate() {
                let field = variant.field_at(i).unwrap();
                dynamic_tuple.insert_boxed(reflect_from_bsn_expr(
                    arg,
                    field.type_info().unwrap(),
                    registry,
                )?);
            }

            Box::new(DynamicEnum::new(
                &variant_name,
                DynamicVariant::Tuple(dynamic_tuple),
            ))
        }
        Expr::Struct(_) if kind == ReflectKind::Enum => {
            // TODO: Enum (struct-like)
            return Err(FromBsnError::Custom(
                "Struct-like enum not supported yet".into(),
            ));
        }
        Expr::Call(_) => {
            // TODO: FunctionRegistry
            return Err(FromBsnError::Custom(
                "Function call not supported yet".into(),
            ));
        }
        _ => {
            return Err(FromBsnError::Custom(
                format!("No registered ReflectFromBsn for type `{}`", ty.type_path()).into(),
            ));
        }
    };

    Ok(val)
}
