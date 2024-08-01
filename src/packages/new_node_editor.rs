use std::any::Any;
use std::fmt::Debug;
use egui::{Color32, Context, Frame, Label, Pos2, Rect, Sense, Stroke, Style, Ui, Vec2, Window};
use crate::if_is_type;
use crate::utility::functions::get_mouse_position;

pub struct NewNodeEditor {
   nodes: Vec<Box<dyn Node>>,
   current_size: Vec2,
   line_state: LineState,
}
impl NewNodeEditor {
   pub fn new() -> Self {
      Self {
         nodes: vec![
            Box::new(Union::new()),
            Box::new(Shape::new()),
         ],
         current_size: Vec2::new(400.0, 400.0),
         line_state: Default::default(),
      }
   }

   pub fn ui(&mut self, context: &Context) {
      let window = Window::new("NewNodeEditor")
          .default_open(false)
          .resizable(true)
          .fixed_size(self.current_size)
          .frame(Frame::window(&Style::default()));

      window.show(&context, |ui| {
         let anchor = ui.min_rect().min;


         for (i, node) in self.nodes.iter_mut().enumerate() {
            ui.push_id(i, |ui| {
               self.line_state.i = i;
               node.show(ui, anchor.to_vec2(), &mut self.line_state);
            });
         }

         if self.line_state.released {
            let o = self.line_state.origin;
            let t = self.line_state.above;

            let no = &self.nodes[o];
            let mut no: Box<dyn Node> = no.clone();
            let nt = &self.nodes[t];

            if no.try_attachment(nt) {
               self.nodes[o] = no;
               self.nodes.remove(t);
               println!("{:?}", &self.nodes[o])
            }

            self.line_state.released = false;
            self.line_state.dragging = false;
         }

         // let mut u = Box::new(Union::new());
         // let mut ou = Box::new(Union::new());
         // let mut s = Box::new(Shape::new());

         // let mut u: Box<dyn Node> = Box::new(Union::new());
         // let mut ou: Box<dyn Node> = Box::new(Union::new());
         // let mut s: Box<dyn Node> = Box::new(Shape::new());
         //
         // if u.try_attachment(&ou) {
         //    // println!("{:?}", u);
         // }

         // let n0 = &self.nodes[0];
         // let mut n0c: Box<dyn Node> = n0.clone();
         //
         // let n1 = self.nodes.get(1).unwrap();
         //
         // if n0c.try_attachment(n1) {
         //    println!("{:?}", n0c)
         // }
         //
         // self.nodes[0] = n0c;


         ui.allocate_space(ui.available_size());
      });

   }
}



///////////
// Nodes //
///////////
#[derive(Clone, Debug)]
struct Union {
   editor_data: EditorData,
   shape: Option<Shape>,
}
impl Union {
   fn new() -> Self {
      Self {
         editor_data: EditorData::new(Vec2::new(75.0, 25.0)),
         shape: None
      }
   }
}
impl Node for Union {
   fn id(&self) -> i32 {
      0
   }

   fn show(&mut self, ui: &mut Ui, anchor: Vec2, line_state: &mut LineState) {
      let handle = Rect::from_min_size((anchor + self.editor_data.position.to_vec2()).to_pos2(), self.editor_data.size);

      // ui code
      ui.allocate_ui_at_rect(handle, |ui| {

         ui.group(|ui| {

            ui.horizontal(|ui| {
               ui.add_space(10.0);
               ui.add(Label::new("Union").selectable(false));
            });

            ui.add(Label::new("1").selectable(false));
            ui.add(Label::new("2").selectable(false));
            ui.add(Label::new("3").selectable(false));

         });

      });


      ui.painter().rect_filled(handle, 2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 20));
      let mut response = ui.allocate_rect(handle, Sense::click_and_drag());
      if response.dragged() {
         self.editor_data.position.x += response.drag_delta().x;
         self.editor_data.position.y += response.drag_delta().y;
      }


      RopeHandle::show(&handle, Pos2::new(50.0, 15.0), line_state, ui);
   }

   fn as_any(&self) -> &dyn Any {
      self
   }

   fn try_attachment(&mut self, other: &Box<dyn Node>) -> bool {
      match &self.shape {
         None => {
            if let Some(connected_shape) = other.as_any().downcast_ref::<Shape>() {
               {
                  println!("Connected");
                  self.shape = Some(connected_shape.clone());
                  return true;
               }
            }
         }
         Some(_) => {}
      }

      false
   }
}



#[derive(Clone, Debug)]
struct Shape {
   editor_data: EditorData,
}
impl Shape {
   fn new() -> Self {
      Self {
         editor_data: EditorData::new(Vec2::new(75.0, 25.0)),
      }
   }

   // fn try_attachment(&mut self, other: &dyn Attachable) -> bool {
   //    println!("Not implemented");
   //    false
   // }
}
impl Node for Shape {
   fn id(&self) -> i32 {
      0
   }

   fn show(&mut self, ui: &mut Ui, anchor: Vec2, line_state: &mut LineState) {
      let handle = Rect::from_min_size((anchor + self.editor_data.position.to_vec2()).to_pos2(), self.editor_data.size);

      // ui code
      ui.allocate_ui_at_rect(handle, |ui| {

         ui.group(|ui| {

            ui.horizontal(|ui| {
               ui.add_space(10.0);
               ui.add(Label::new("Shape").selectable(false));
            });

            ui.add(Label::new("1").selectable(false));
            ui.add(Label::new("2").selectable(false));
            ui.add(Label::new("3").selectable(false));

         });

      });


      ui.painter().rect_filled(handle, 2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 20));
      let mut response = ui.allocate_rect(handle, Sense::click_and_drag());
      if response.dragged() {
         self.editor_data.position.x += response.drag_delta().x;
         self.editor_data.position.y += response.drag_delta().y;
      }

      RopeHandle::show(&handle, Pos2::new(50.0, 15.0), line_state, ui);
   }

   fn as_any(&self) -> &dyn Any {
      self
   }

   fn try_attachment(&mut self, other: &Box<dyn Node>) -> bool {
      // if let Some(shape) = other.as_any().downcast_ref::<Shape>() {
      //    {
      //       println!("Connected");
      //       return true;
      //    }
      // }
      println!("Not Implemented");
      false
   }
}





//////////////////////
// Non node structs //
//////////////////////
#[derive(Clone, Debug)]
struct EditorData {
   position: Pos2,
   size: Vec2,
}
impl EditorData {
   fn new(size: Vec2) -> Self {
      Self {
         position: Pos2::new(0.0, 0.0),
         size,
      }
   }
}


#[derive(Debug, Default)]
struct LineState {
   dragging: bool,
   origin: usize,

   above: usize,
   released: bool,

   i: usize,

}


struct RopeHandle {}
impl RopeHandle {
   fn show(reference: &Rect, pos: Pos2, line_state: &mut LineState, ui: &mut Ui) {
      let circle_pos = pos + reference.min.to_vec2();
      ui.painter().circle_filled(circle_pos, 7.0, Color32::from_rgb(255, 0, 0));
      let mut cable_response = ui.allocate_rect(Rect::from_center_size(circle_pos, Vec2::new(7.0, 7.0)), Sense::click_and_drag());

      if cable_response.hovered() {
         line_state.above = line_state.i;
         ui.painter().circle_filled(circle_pos, 3.0, Color32::from_rgb(255, 255, 255));
      }

      if cable_response.drag_started() {
         line_state.dragging = true;
         line_state.origin = line_state.i;
      }

      if cable_response.drag_stopped() {
         line_state.released = true;
      }

      if cable_response.dragged() {
         ui.painter()
            .line_segment([circle_pos, get_mouse_position(ui.ctx())], Stroke::new(3.0, Color32::WHITE));
      }

   }
}


////////////
// traits //
////////////
trait Node: CloneBox + Debug {
   fn id(&self) -> i32;
   fn show(&mut self, ui: &mut Ui, anchor: Vec2, line_state: &mut LineState);
   fn as_any(&self) -> &dyn Any;
   fn try_attachment(&mut self, other: &Box<dyn Node>) -> bool;
}

impl Clone for Box<dyn Node> {
   fn clone(&self) -> Box<dyn Node> {
      self.clone_box()
   }
}

trait CloneBox {
   fn clone_box(&self) -> Box<dyn Node>;
}

impl<T: ?Sized> CloneBox for T
where
    T: 'static + Node + Clone,
{
   fn clone_box(&self) -> Box<dyn Node> {
      Box::new(self.clone())
   }
}