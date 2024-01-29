struct Sedan;

impl LandCapable for Sedan {
    fn drive(&self) {
        println!("Sedan is driving");
    }
}

struct SUV;
impl LandCapable for SUV {
    fn drive(&self) {
        println!("SUV is driving");
    }
}

struct Bicycle;
impl LandCapable for Bicycle {

}

trait LandCapable {
    fn drive(&self) {
        println!("Default impl drive")
    }
}

trait WaterCapable {
    fn float(&self) {
        println!("Default impl float")
    }
}

trait Amphibious : WaterCapable + LandCapable {
}

struct Hovercraft;
impl Amphibious for Hovercraft {}
impl LandCapable for Hovercraft {
    fn drive(&self) {
        println!("Hovercraft is DRIVING")
    }
}
impl WaterCapable for Hovercraft {
    fn float(&self) {
        println!("Hovercraft is FLOATING")
    }
}

fn road_trip(vehicle: &impl LandCapable) {
    vehicle.drive();
}

fn traverse_frozen_lake(vehicle: &impl Amphibious) {
    vehicle.drive();
    vehicle.float();
}

fn main() {
    let hc = Hovercraft;
    traverse_frozen_lake(&hc);

    let car = Sedan;
    road_trip(&car);

    let suv = SUV;
    road_trip(&suv);

    let bike = Bicycle;
    road_trip(&bike);
}
