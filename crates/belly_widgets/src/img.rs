use belly_core::build::*;
use belly_macro::*;

use bevy::{
    asset::Asset,
    prelude::*,
    utils::{HashMap, HashSet},
};
use std::str::FromStr;

pub mod prelude {
    pub use super::Img;
    pub use super::ImgEvent;
    pub use super::ImgMode;
    pub use super::ImgWidgetExtension;
}

pub(crate) struct ImgPlugin;
impl Plugin for ImgPlugin {
    fn build(&self, app: &mut App) {
        app.register_widget::<ImgWidget>();

        app.init_resource::<ImageRegistry>();
        app.add_system(load_img.label(ImgLabel::Load));
        app.add_system(
            update_img_size
                .label(ImgLabel::UpdateSize)
                .after(ImgLabel::Load),
        );
        app.add_system(
            update_img_layout
                .label(ImgLabel::UpdatLayout)
                .after(ImgLabel::UpdateSize),
        );
        app.add_event::<ImgEvent>();
    }
}

#[widget]
#[signal(load:ImgEvent => |e| e.loaded())]
#[signal(unload:ImgEvent => |e| e.unloaded())]
/// Specifies the path to the image or custom `Handle<Image>`
#[param( src: ImageSource => Img:src )]
/// <!-- @inline ImgMode -->
#[param( mode: ImgMode => Img:mode )]
/// Specifies the color the image should be multiplied
#[param( modulate: Color => Img:modulate )]
/// The `<img>` is used to load image and show it content on the UI screen.
fn img(ctx: &mut WidgetContext, img: &mut Img) {
    let this = ctx.entity();
    let content = ctx.content();
    ctx.add(from!(this, Img: modulate) >> to!(img.entity, BackgroundColor:0));
    ctx.commands().entity(img.entity).insert(ImageBundle {
        style: Style {
            display: Display::None,
            ..default()
        },
        ..default()
    });
    ctx.insert(ElementBundle::default())
        .push_children(&[img.entity]);
    ctx.commands().entity(img.entity).push_children(&content);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum ImgLabel {
    Load,
    UpdateSize,
    UpdatLayout,
}

#[derive(Resource, Deref, DerefMut, Default)]
struct ImageRegistry(HashMap<Handle<Image>, HashSet<Entity>>);

// #[derive(Even)]
pub enum ImgEvent {
    Loaded(Vec<Entity>),
    Unloaded(Vec<Entity>),
}

impl ImgEvent {
    pub fn loaded(&self) -> bool {
        match self {
            ImgEvent::Loaded(_) => true,
            ImgEvent::Unloaded(_) => false,
        }
    }

    pub fn unloaded(&self) -> bool {
        match self {
            ImgEvent::Loaded(_) => false,
            ImgEvent::Unloaded(_) => true,
        }
    }
}

impl Signal for ImgEvent {
    fn sources(&self) -> &[Entity] {
        match self {
            ImgEvent::Loaded(entities) => &entities,
            ImgEvent::Unloaded(entities) => &entities,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
/// Specifies how an image should fits the space:
/// - `fit`: resize the image to fit the box keeping it aspect ratio
/// - `cover`: resize the image to cover the box keeping it aspect ratio
/// - `stretch`: resize image to take all the space ignoring the aspect ratio
/// - `source`: keep image at original size
pub enum ImgMode {
    #[default]
    Fit,
    Cover,
    Stretch,
    Source,
}

impl FromStr for ImgMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(ImgMode::Fit),
            "fit" => Ok(ImgMode::Fit),
            "cover" => Ok(ImgMode::Cover),
            "stretch" => Ok(ImgMode::Stretch),
            "source" => Ok(ImgMode::Source),
            err => Err(format!("Can't parse `{}` as ImgMode", err)),
        }
    }
}

impl TryFrom<String> for ImgMode {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<Variant> for ImgMode {
    type Error = String;
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        value.get_or_parse()
    }
}

impl From<ImgMode> for Variant {
    fn from(mode: ImgMode) -> Self {
        Variant::Boxed(Box::new(mode))
    }
}

#[derive(Clone)]
pub enum AssetSource<T: Asset> {
    Path(String),
    Handle(Handle<T>),
}

pub type ImageSource = AssetSource<Image>;

impl<T: Asset> Default for AssetSource<T> {
    fn default() -> Self {
        Self::Path("".into())
    }
}

impl<T: Asset> PartialEq for AssetSource<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Path(s), Self::Path(o)) => s == o,
            (Self::Handle(s), Self::Handle(o)) => s == o,
            _ => false,
        }
    }
}

impl<T: Asset> From<String> for AssetSource<T> {
    fn from(s: String) -> Self {
        AssetSource::Path(s)
    }
}

impl<T: Asset> From<Handle<T>> for AssetSource<T> {
    fn from(h: Handle<T>) -> Self {
        AssetSource::Handle(h)
    }
}

impl<T: Asset> TryFrom<Variant> for AssetSource<T> {
    type Error = String;
    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match value {
            Variant::String(s) => Ok(AssetSource::Path(s)),
            Variant::Boxed(h) if h.is::<String>() => {
                Ok(AssetSource::Path(match h.downcast::<String>() {
                    Ok(path) => *path,
                    Err(e) => return Err(format!("Cant convert '{e:?}' to AssetSource")),
                }))
            }
            Variant::Boxed(h) if h.is::<Handle<T>>() => {
                Ok(AssetSource::Handle(match h.downcast::<Handle<T>>() {
                    Ok(handle) => *handle,
                    Err(e) => return Err(format!("Cant convert '{e:?}' to AssetSource")),
                }))
            }
            e => Err(format!("Cant convert '{:?}' to AssetSource", e)),
        }
    }
}

#[derive(Component)]
pub struct Img {
    pub src: AssetSource<Image>,
    pub mode: ImgMode,
    pub modulate: Color,
    handle: Handle<Image>,
    entity: Entity,
    size: Vec2,
}

impl FromWorldAndParams for Img {
    fn from_world_and_params(world: &mut World, params: &mut belly_core::eml::Params) -> Self {
        Img {
            src: params.try_get("src").unwrap_or_default(),
            mode: params.try_get("mode").unwrap_or_default(),
            modulate: params.try_get("modulate").unwrap_or_default(),
            handle: Default::default(),
            entity: world.spawn_empty().id(),
            size: Default::default(),
        }
    }
}

fn load_img(
    asset_server: Res<AssetServer>,
    mut elements: Query<(Entity, &mut Img), Changed<Img>>,
    mut images: Query<(&mut UiImage, &mut Style)>,
    mut registry: ResMut<ImageRegistry>,
    assets: Res<Assets<Image>>,
    mut events: EventWriter<AssetEvent<Image>>,
    mut signals: EventWriter<ImgEvent>,
) {
    for (entity, mut img) in elements.iter_mut() {
        let handle = match &img.src {
            AssetSource::Path(s) if s.is_empty() => Handle::default(),
            AssetSource::Path(s) => asset_server.load(s),
            AssetSource::Handle(h) => h.clone(),
        };
        if handle != img.handle {
            if assets.contains(&img.handle) {
                signals.send(ImgEvent::Unloaded(vec![entity]));
            }
            registry
                .entry(img.handle.clone_weak())
                .or_default()
                .remove(&entity);
            registry
                .entry(handle.clone_weak())
                .or_default()
                .insert(entity);
            img.handle = handle.clone();
        }
        let (mut image, mut style) = images.get_mut(img.entity).unwrap();
        image.0 = handle.clone();

        // force inner image size recalculation if Image asset already loaded
        if assets.contains(&handle) {
            style.display = Display::Flex;
            events.send(AssetEvent::Modified {
                handle: handle.clone_weak(),
            });
            signals.send(ImgEvent::Loaded(vec![entity]));
        } else {
            if img.size != Vec2::ZERO {
                img.size = Vec2::ZERO;
            }
            style.display = Display::None;
        }
    }
}

fn update_img_size(
    mut elements: Query<&mut Img>,
    assets: Res<Assets<Image>>,
    mut asset_events: EventReader<AssetEvent<Image>>,
    mut registry: ResMut<ImageRegistry>,
) {
    for event in asset_events.iter() {
        match event {
            AssetEvent::Removed { handle } => {
                let Some(entities) = registry.remove(&handle) else { continue };
                for entity in entities.iter() {
                    let Ok(mut element) = elements.get_mut(*entity) else { continue };
                    element.size = Vec2::ZERO;
                }
            }
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                let Some(entities) = registry.get(&handle) else { continue };
                for entity in entities.iter() {
                    let Ok(mut element) = elements.get_mut(*entity) else { continue };
                    let Some(asset) = assets.get(handle) else { continue };
                    element.size = asset.size();
                }
            }
        }
    }
}

fn update_img_layout(
    elements: Query<(&Img, &Node), Or<(Changed<Img>, Changed<Node>)>>,
    mut styles: Query<&mut Style>,
) {
    for (element, node) in elements.iter() {
        let Ok(mut style) = styles.get_mut(element.entity) else { continue };
        if element.size.x.abs() < f32::EPSILON
            || element.size.y.abs() < f32::EPSILON
            || node.size().x.abs() < f32::EPSILON
            || node.size().y.abs() < f32::EPSILON
        {
            style.display = Display::None;
            continue;
        } else {
            style.display = Display::Flex;
        }
        let aspect = element.size.y / element.size.x;
        match element.mode {
            ImgMode::Fit => {
                let (width, height) = if aspect > 1.0 {
                    let width = node.size().x;
                    let height = width * aspect;
                    if height > node.size().y {
                        let width = width * (node.size().y / height);
                        let height = node.size().y;
                        (width, height)
                    } else {
                        (width, height)
                    }
                } else {
                    let height = node.size().y;
                    let width = height / aspect;
                    if width > node.size().x {
                        let height = height * (node.size().x / width);
                        let width = node.size().x;
                        (width, height)
                    } else {
                        (width, height)
                    }
                };
                style.min_size.height = Val::Px(height);
                style.min_size.width = Val::Px(width);
                style.size = style.min_size;
                let hmargin = 0.5 * (node.size().x - width);
                let vmargin = 0.5 * (node.size().y - height);

                style.margin.top = Val::Px(vmargin.max(0.));
                style.margin.bottom = Val::Px(vmargin.max(0.));
                style.margin.left = Val::Px(hmargin.max(0.));
                style.margin.right = Val::Px(hmargin.max(0.));
            }
            ImgMode::Cover => {
                let (width, height) = if aspect > 1.0 {
                    let width = node.size().x;
                    let height = width * aspect;
                    if height < node.size().y {
                        let width = width * (node.size().y / height);
                        let height = node.size().y;
                        (width, height)
                    } else {
                        (width, height)
                    }
                } else {
                    let height = node.size().y;
                    let width = height / aspect;
                    if width < node.size().x {
                        let height = height * (node.size().x / width);
                        let width = node.size().x;
                        (width, height)
                    } else {
                        (width, height)
                    }
                };

                style.min_size.height = Val::Px(height);
                style.min_size.width = Val::Px(width);
                style.size = style.min_size;
                let hmargin = 0.5 * (node.size().x - width);
                let vmargin = 0.5 * (node.size().y - height);

                style.margin.top = Val::Px(vmargin.min(0.));
                style.margin.bottom = Val::Px(vmargin.min(0.));
                style.margin.left = Val::Px(hmargin.min(0.));
                style.margin.right = Val::Px(hmargin.min(0.));
            }
            ImgMode::Stretch => {
                style.min_size = Size::new(Val::Undefined, Val::Undefined);
                style.size = Size::new(Val::Percent(100.), Val::Percent(100.));
                style.margin = UiRect::all(Val::Px(0.));
            }
            ImgMode::Source => {
                style.size = Size::new(Val::Px(element.size.x), Val::Px(element.size.y));
                style.min_size = style.size;
                let hmargin = 0.5 * (node.size().x - element.size.x);
                let vmargin = 0.5 * (node.size().y - element.size.y);
                style.margin.left = Val::Px(hmargin);
                style.margin.right = Val::Px(hmargin);
                style.margin.top = Val::Px(vmargin);
                style.margin.bottom = Val::Px(vmargin);
            }
        }
    }
}
