enum Point {
    Cartesian { x: f32, y: f32 },
    Polar { r: f32, theta: f32 },
}

enum Shape {
    Circle { center: Point, radius: f32 },
    Rect,
}

fn main() {
    let shape = Shape::Circle {
        center: Point::Polar {
            r: 10.0,
            theta: 1.57,
        },
        radius: 5.0,
    };

    match shape {
        Shape::Circle { center, radius } => {
            println!("半径: {}", radius);

            match center {
                Point::Cartesian { x, y } => {
                    println!("笛卡尔坐标 ({}, {})", x, y);
                }
                Point::Polar { theta, .. } => {
                    println!("夹角: {}", theta);
                }
            }
        }
        Shape::Rect => {}
    }
}
