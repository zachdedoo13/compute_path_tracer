use std::any::Any;
use compute_path_tracer::if_is_type;
use compute_path_tracer::inbuilt::event_loop::run;

trait Node {
    fn id(&self) -> i32;
}

trait Attachable: Any + Node {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Node> Attachable for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
struct Shape {
    id: i32,
}

#[derive(Clone)]
struct OSS {
    id: i32,
    fkuke: Vec<f32>,
    sh: Option<Shape>,
}

impl Node for Shape {
    fn id(&self) -> i32 {
        self.id
    }
}

impl Node for OSS {
    fn id(&self) -> i32 {
        self.id
    }
}

struct Union {
    sub: Option<Shape>,
}

impl Union {
    fn attach_if_type(&mut self, other: &Box<dyn Attachable>) -> bool {
        println!("Trying to attach node with id: {}", other.id());
        // if let Some(shape) = other.as_any().downcast_ref::<Shape>() {
        //     self.sub = Some(shape.clone());
        //     println!("Connected");
        //     return true;
        // }

        if_is_type!(shape, other, Shape, {
            self.sub = Some(shape.clone());
            println!("Connected");
            return true;
        });

        println!("Incorrect type");

        false
    }
}



fn main() {
    // let shape: Box<dyn Attachable> = Box::new(Shape { id: 1 });
    //
    // println!("{}", shape.id());
    // let mut union = Union { sub: None };
    //
    // //let oss = OSS { id: 2, fkuke: vec![12.0, 124124.0], sh: Some(Shape { id: 0 }) };
    //
    // union.attach_if_type(&shape);
    // //union.attach_if_type(&oss);
    

    pollster::block_on(run());
}