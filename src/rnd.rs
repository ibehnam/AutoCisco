enum Employee {
    Manager {name: String, subordinates: Vec<Box<Employee>>},
    Worker {name: String, manager: String},
}

fn main() {
    let employee = Employee::
    println!("Hello, world!")
}