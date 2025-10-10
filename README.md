# Usage

A simple macro to help define rpc send and receive protocols. 

The `rpc_defer!` macro converts the function call into a 
struct representing the function call. This struct can be 
encoded and sent over the wire to another machine, where it 
can be called via `rpc_call!`.

```rust
use rpc_macro::{rpc_functions, rpc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u8,
    phones: Vec<String>,
}

rpc_functions! {
    #[rpc] fn say_hello(person: Person) -> u32 {
        println!("Hi {}!", person.name);
        10
    }

    #[rpc] fn say_goodbye(person: Person) -> u32 {
        println!("Bye {}!", person.name);
        10
    }

    #[rpc] fn greet_person(person: Person, other_person: Person) -> u32 {
        println!("Hi, my name is {}! Nice to meet you {}!", person.name, other_person.name);
        10
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = Person {
        name: "Sid".to_string(),
        age: 22,
        phones: vec!["+1 (XXX) XXX XXXX".to_string()],
    };

    let _result = rpc_call!{rpc_defer!{ say_hello(p.clone()) }};
    rpc_call!( rpc_defer!{ say_goodbye(p.clone())             });
    rpc_call!( rpc_defer!{ greet_person(p.clone(), p.clone()) });

    Ok(())
}
```
