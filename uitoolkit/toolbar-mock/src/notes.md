A traditional UI toolkit is hard to implement in Rust for a couple of reasons.

First, most UI toolkits assume some sort of a base widget class that all widgets descend from using
inheritance. They override just specific methods of the base class to become the many variations of widgets.
This won't work in Rust because it doesn't have implementation inheritance.

Second, UI toolkits usually create a tree structure of widgets, and this structure can change at runtime
based on what the application is doing. Dynamic tree structures are hard to build in Rust because Rust
really wants to know who owns what.

Third, UI toolkits usually have a spagetti ball of event handlers all over the place, used to connect
the widget tree to whatever the application specific data model is.  Again, this is hard to manage in Rust
becauwe we must track ownership instead of letting a Garbage Collector sort it all out.

So what can we do?

First, lets break up the widget class. Instead of a single base class (or interface) that all widgets must
implement, split it up int chunks. A widget is just a composition of those chunks. Some of these chunks
can be optional. there could be an input chunk that a label wouldn't need because it doesn't accept input. We
can put all of the mandatory state that all widgets must have (visible, id, name, disabled, etc) into a standard
struct so that it doesn't have to be written multiple times.  

A typical widget base interface has a bunch of methods but they can largely be grouped into the following:

* standard state (enabled, visible, focused, active, hovered, id, name, etc.)
* widget specific state (togglebutton.selected, selection.data_list, textfield.text, etc.)
* receiving and generating events
* layout (geometry and positioning)
* drawing to the screen

Let's split each of these areas out into traits with standard implementations. If you need to go custom then you only need to
do a new implementation of just that part, not everything.

Second, let's move as much common state out of the structs themselves into parameters passed to the traits. This
includes things like the current stylesheet, the current language settings and translations, the widget
with the current keyboard focus, the current window, and anything else that is likely to be shared with lots of widgets.




One thing I really don't like about the Rust UI toolkits i've seen is they use lots of magic macros to generate
code. It doesn't smell right to me. It's not very rusty.


# test case

Let's mockup an itunes like app: three horizontal rows: toolbar, content area, and statusbar. The content
area has a list of music sources (by artist, by album, by song, all podcasts, playlists, etc). The user
chooses a source by clicking on it. Then the other part of the content area shows the selected list in
the correct view. ex: selecting artists could show a list of artists with the albums next to them, but selecting
by albums would show album cover artwork. Selecting 'by song' would show a full sortable and filterable table view.
Searching from the text input in the toolbar would also replace the main content area.
The toolbar area has prev,play/pause,next buttons, a spacer, a currently playing custom component, a spacer, and
the search field.  The status bar will show information about what is currently selected in the main area. 

Let's consider the data structure for this application. 

```
Database:
    queryByTitle
    queryByArtist
    queryByAlbum
    queryByType(type:podcast)
    queryByPlaylist()
```

* A list of music sources defined in terms of the database above: `sources:Vec<MusicSource>`
* The currently selected music source: `selectedSource:MusicSource`
* The query in the search field: `searchQuery:String`
* The currently playing track: `currentTrack:MusicTrack`
* The current track list (meaning what will be played before and 
  after the current song, not necessarily what the user currently
  has selected): `currentTrackList:Vec<MusicTrack>`
* the metadata for the currently playing track (title, artist, album, duration) `strings and numbers`
* playback settings (repeat, random shuffle, single repeat): `enum RepeatType { Repeat, Shuffle, SingleRepeat }`
* playback volume `f32`

Now lets create the high level widgets we need:

```
window:VBox
    toolbar:HBox (hflex:true, vflex:false)
        prev:IconButton
        playpause:IconToggleButton
        next:IconButton
        spacer:HSpacer
        currentTrack:CustomDisplay
        spacer:HSpacer
        search:SingleLineTextEdit
    centerRow:HBox (hflex:true, vflex:true)
        sources:SelectList (hflex:false, vflex:true)
        results:ScrollView (hflex:true, vflex:true)  (contents will change periodically)
    statusBar:HBox (hflex:true, vflex:false)
        currentSong:Label
        spacer:HSpacer
        selectionStatus:Label
```

Now let's try to create just the toolbar in Rust code just with nested structs, splitting as advised before:

```rust
fn main() {
  let toolbar = Container {
    state:StandardState::new().with_name("toolbar"),
    input:None,
    layout:HBoxLayout::new(),
    paint:None,
    children: vec![
      View {
        name:"prev",
        state:StandardState::new(),
        input:ButtonInput::new().on_action(|e| state.current_tracklist.nav_prev_track()),
        layout:ButtonLayout::new(),
        paint:ButtonPaint::new(),
      },
      View {
        state:StandardState::new().with_name("playpause"),
        input:ButtonInput::new().on_action(|e| state.current_tracklist.toggle_playing_current_track()),
        layout:ButtonLayout::new(),
        paint:ButtonPaint::new(),
      },
      View {
        name:"spacer1",
        state:StandardState::new(),
        layout:HSpacerLayout::new(),
        paint:None,
        input:None,
      },
      View {
        name:"currentTrackDisplay",
        state:StandardState::new(),
        layout:FixedSizeLayout::new(200,100),
        paint:CustomTrackDisplay::new(),
        input:None,
      },
      View {
        name:"spacer2",
        state:StandardState::new(),
        layout:HSpacerLayout::new(),
        paint:None,
        input:None,
      },
      View {
        name:"search",
        state:StandardState::new(),
        layout:TextLineLayout::new(),
        paint:TextLinePaint::new(),
        input:TextLineInput::new(),
      },
    ]
  };
}

```


Looking at the above we can learn a few things. First, it's rather verbose but pretty straightforward. We could
easily create helper functions for common widgets like buttons and spacers so that what you need to write
out would be a lot shorter.  

Second we can see that any given widget really is a composite of several traits which can easily be swapped out
with new ones. That's cool.  Now lets look at the traits themselves.

Consider the Label widget. It takes no input and really has no state to speak of other than the text it displays.
However this text can change at runtime.  It's layout should consist of measuring the text in the current font and
adding some spacing on the sides.  Painting should just be drawing that one line of text.  So let's see what it
takes to build it.

First we need the standard state that all widgets have. For now let's pretend it's just whether the
widget is enabled or not. Then we can make a LabelState struct to hold the text and combine them by
implementing StandardState on the LabelState:

```rust
trait StandardState {
  fn enabled(&self) -> bool;
  fn set_enabled(&self, enabled:bool);
}

struct Label {
  text:String,
  _enabled:bool,
}
impl StandardState for Label {
  fn enabled(&self) -> bool {
    return self._enabled
  }
}
```


Now we need to be able to layout the label, so we need to implement a Layout trait:
```rust
trait Layout {
  fn layout(&self, layout:&LayoutContext, state:&StandardState) -> Size;
}
impl Layout for Label {
  fn layout(&self, layout: &LayoutContext) {
    let size = layout.measureText(self.text).grow(layout.standardLabelPadding());
    return size;
  }
}
```

Now we need to draw the label. Let's add the Paint trait
```rust
trait Paint {
  fn paint(&self, paint:&PaintContext);
}
impl Paint for Label {
  fn paint(&self, paint: &PaintContext) {
    paint.fillText(self.text, paint.standardLabelOffset(),paint.standardLabelTextColor());
  }
}
```





