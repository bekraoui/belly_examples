// examples/tabview.rs
// cargo run --example tabview
use belly::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BellyPlugin)
        .add_event::<NewTab>()
        .add_startup_system(setup)
        .add_system(add_tab)
        .run();
}

#[derive(Component, Default)]
struct IsTab;

// event to triger create new editor
pub struct NewTab {
    tabs: Entity,
    tabs_content: Entity,
}

#[derive(Component, Default)]
struct TabController;
fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    let tabs = commands.spawn_empty().id();
    let tabs_content = commands.spawn_empty().id();

    commands.add(StyleSheet::parse(
        "
        .hidden {
            display: none;
        }
    ",
    ));
    commands.add(eml! {
      <body s:padding="20px">
        <buttongroup on:value_change=connect!(|ctx| {
            let ev = ctx.event();
            ctx.select(ev.old_value()).add_class("hidden");
            ctx.select(ev.new_value()).remove_class("hidden");
        })>
          <span {tabs}>
            <button with=IsTab value=".tab1" pressed>"Tab 1"</button>
            <button with=IsTab value=".tab2">"Tab 2"</button>
            <button with=IsTab value=".tab3">"Tab 3"</button>
          </span>
        </buttongroup>
        //refactor this
        <button value=".add_tab" on:press=connect!(|ctx| {
          ctx.send_event(NewTab{
                            tabs         : tabs,
                            tabs_content : tabs_content,
                          })
        })> "+" </button>
        <br/>
        <div c:content {tabs_content}>
          <div c:tab1>"Tab 1 content"</div>
          <div c:tab2 c:hidden>"Tab 2 content"</div>
          <div c:tab3 c:hidden>"Tab 3 content"</div>
        </div>
      </body>
    });
}

fn add_tab(mut commands: Commands, mut events: EventReader<NewTab>, query: Query<&IsTab>) {
    for event in events.iter() {
        //add the new tab button to <span {tabs}>
        let tab = commands.spawn(IsTab).id();
        let count = query.iter().count() + 1;
        commands.add(eml! {
          <button {tab} value=format!(".tab{}", count)>
              { format!("Tab {}", count) }
          </button>
        });
        commands.entity(event.tabs).push_children(&[tab]);

        //add the new tab content to tabs_content <div>
        let content = commands.spawn_empty().id();
        commands.add(eml! {
          <div {content} class=format!("tab{}", count) c:hidden>
          { format!("Tab {} content", count) }
          </div>
        });
        commands
            .entity(event.tabs_content)
            .push_children(&[content]);
    }
}
