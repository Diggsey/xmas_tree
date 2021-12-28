use bevy::{
    prelude::*,
    render::{
        pass::{
            LoadOp, Operations, PassDescriptor, RenderPassDepthStencilAttachmentDescriptor,
            TextureAttachment,
        },
        render_graph::{base, PassNode, RenderGraph, WindowSwapChainNode, WindowTextureNode},
    },
};

#[derive(Debug, Default)]
pub struct AlwaysOnTopPlugin;

impl Plugin for AlwaysOnTopPlugin {
    fn build(&self, app: &mut AppBuilder) {
        add_aot_graph(app.world_mut());
    }
}

#[derive(Default)]
pub struct AlwaysOnTopPass;

pub const ALWAYS_ON_TOP_PASS: &str = "aot_pass";

fn add_aot_graph(world: &mut World) {
    let world = world.cell();
    let mut graph = world.get_resource_mut::<RenderGraph>().unwrap();
    let msaa = world.get_resource::<Msaa>().unwrap();

    let mut aot_pass_node = PassNode::<&AlwaysOnTopPass>::new(PassDescriptor {
        color_attachments: vec![msaa.color_attachment_descriptor(
            TextureAttachment::Input("color_attachment".to_string()),
            TextureAttachment::Input("color_resolve_target".to_string()),
            Operations {
                load: LoadOp::Load,
                store: true,
            },
        )],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
            attachment: TextureAttachment::Input("depth".to_string()),
            depth_ops: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        }),
        sample_count: msaa.samples,
    });
    aot_pass_node.add_camera(base::camera::CAMERA_3D);

    graph.add_node(ALWAYS_ON_TOP_PASS, aot_pass_node);

    graph
        .add_slot_edge(
            base::node::PRIMARY_SWAP_CHAIN,
            WindowSwapChainNode::OUT_TEXTURE,
            ALWAYS_ON_TOP_PASS,
            if msaa.samples > 1 {
                "color_resolve_target"
            } else {
                "color_attachment"
            },
        )
        .unwrap();

    graph
        .add_slot_edge(
            base::node::MAIN_DEPTH_TEXTURE,
            WindowTextureNode::OUT_TEXTURE,
            ALWAYS_ON_TOP_PASS,
            "depth",
        )
        .unwrap();

    if msaa.samples > 1 {
        graph
            .add_slot_edge(
                base::node::MAIN_SAMPLED_COLOR_ATTACHMENT,
                WindowSwapChainNode::OUT_TEXTURE,
                ALWAYS_ON_TOP_PASS,
                "color_attachment",
            )
            .unwrap();
    }

    // ensure AOT pass runs after main pass
    graph
        .add_node_edge(base::node::MAIN_PASS, ALWAYS_ON_TOP_PASS)
        .unwrap();
}
