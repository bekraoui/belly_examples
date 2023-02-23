// examples/multi-system-on-event.rs
// cargo run --example multi-system-on-event
use belly::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BellyPlugin)
        .add_event::<SetupChild1>()
        .add_event::<SetupChild2>()
        .add_startup_system(setup_parent)
        .add_system(setup_child_1)
        .add_system(setup_child_2)
        .run();
}

// event to triger new child_1 creation
pub struct SetupChild1 {
    parent: Entity,
}
// event to triger new child_1 creation
pub struct SetupChild2 {
    parent: Entity,
}

fn setup_parent(
    mut commands: Commands,
) {
    commands.add(StyleSheet::parse(
        "
        .img {
            width: 50%;
            height: 50%;
          }
    ",
    ));

    commands.spawn(Camera2dBundle::default());
    let parent_1 = commands.spawn_empty().id();
    let parent_2 = commands.spawn_empty().id();
    commands.add(eml! {
        <body s:padding="50px">
            <div>
                "Hello, "<strong>"world"</strong>"!"
                
                <button value="add_child_1" on:press=connect!(|ctx| {
                    ctx.send_event(SetupChild1 {
                        parent: parent_1,
                    })
                })> "add child 1" </button>

                <button value="add_child_2" on:press=connect!(|ctx| {
                    ctx.send_event(SetupChild2 {
                        parent: parent_2,
                    })
                })> "add child 2" </button>
            </div>
            <div>
                <br/><span {parent_1}> </span>
                <br/><span {parent_2}> </span>
            </div>
        </body>
    });
}

fn setup_child_1(mut commands: Commands, mut ev_setup_child: EventReader<SetupChild1>) {
    for ev in ev_setup_child.iter() {
        let content = commands.spawn_empty().id();
        commands.add(eml! {
            <span {content}>
                <div>" Child 1 content with an image "</div>
                <div><img src="icon.png"/></div>
            </span>
        });
        commands.entity(ev.parent).push_children(&[content]);
    }
}

fn setup_child_2(mut commands: Commands, mut ev_setup_child: EventReader<SetupChild2>) {
    for ev in ev_setup_child.iter() {
        let content = commands.spawn_empty().id();
        commands.add(eml! {
            <span {content}>
                <strong>" Child 2 content with a text "</strong>"!"
            </span>
        });
        commands.entity(ev.parent).push_children(&[content]);
    }
}
