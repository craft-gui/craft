use std::sync::Arc;

use kurbo::Affine;

use retgui_resource_manager::ResourceManager;
use retgui_resource_manager::image::ImageResource;
use retgui_resource_manager::resource::Resource;
use retgui_resource_manager::resource_type::ResourceType;

use vello_common::kurbo;
use vello_common::paint::{ImageId, ImageSource, PaintType};
use vello_common::peniko::ImageAlphaType;
use vello_common::pixmap::{PixelMetadata, Pixmap};

use vello_cpu::{RenderContext, Resources};

use crate::render_command::DrawImageCmd;
use crate::resource_mapper::{RendererResourceId, ResourceMapper};

pub(crate) fn upload_image(
    cmd: &DrawImageCmd,
    resource_manager: Arc<ResourceManager>,
    resources: &mut Resources,
    resource_mapper: &mut ResourceMapper,
) -> Option<RendererResourceId> {
    let resource = resource_manager.get(&cmd.resource_id)?;
    let image = resource_to_image_resource(resource.as_ref())?;

    // TODO: Handle expired images
    let resource_id = if let Some(resource_id) = resource_mapper.get(&cmd.resource_id) {
        resource_id
    } else {
        let pixmap = Pixmap::from_parts(
            image.image.as_raw().clone(),
            image.get_width() as u16,
            image.get_height() as u16,
            PixelMetadata::new(ImageAlphaType::Alpha, true),
        );
        let image_id = resources.register_image(Arc::new(pixmap));
        let renderer_resource_id = RendererResourceId(image_id.as_u32() as u64);

        resource_mapper.add_mapping(cmd.resource_id.clone(), renderer_resource_id.clone());

        renderer_resource_id
    };

    Some(resource_id)
}

pub(crate) fn draw_image(
    cmd: &DrawImageCmd,
    scene: &mut RenderContext,
    resource_manager: Arc<ResourceManager>,
    resource_id: RendererResourceId,
) {
    let Some(resource) = resource_manager.get(&cmd.resource_id) else {
        return;
    };
    let Some(image) = resource_to_image_resource(resource.as_ref()) else {
        return;
    };

    let mut transform = Affine::IDENTITY;
    transform = transform.with_translation(kurbo::Vec2::new(cmd.rect.x as f64, cmd.rect.y as f64));
    transform = transform.pre_scale_non_uniform(
        cmd.rect.width as f64 / image.get_width() as f64,
        cmd.rect.height as f64 / image.get_height() as f64,
    );
    scene.set_transform(cmd.transform * transform);

    let vello_image = vello_common::paint::Image {
        image: ImageSource::OpaqueId {
            id: ImageId::new(resource_id.0 as u32),
            may_have_transparency: true,
        },
        sampler: Default::default(),
    };

    scene.set_paint(PaintType::Image(vello_image));
    scene.fill_rect(&kurbo::Rect::new(
        0.0,
        0.0,
        image.get_width() as f64,
        image.get_height() as f64,
    ));
}

fn resource_to_image_resource(resource: &Resource) -> Option<&ImageResource> {
    if resource.resource_type != ResourceType::Image {
        return None;
    }
    resource.data.downcast_ref::<ImageResource>()
}
