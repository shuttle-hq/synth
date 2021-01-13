# Bynar

![Jeff Bynar](./res/jeff_bynar.jpg)

## What is this?

`bynar` is a Rust crate that lets you generate highly customizable
synthetic populations of arbitrarily complex objects.

It works by generating streams of tokens (called `Bynar`s) that can be
consumed by your structs' `serde::Deserialize` implementations. You
can choose to configure those streams to enforce the relations you
want between fields or objects or even reproduce a desired dummy
population's statistical behavior.

## Just show me an example...

Sure!

```rust
use serde::Deserialize;
use bynar::Bynar;

struct Name(String);

impl Deserialize for Name {
   // [...]
}

struct DateOfBirth(chrono::Date<Utc>);

impl Deserialize for DateOfBirth {
   // [...]
}

struct Email(String);

impl Deserialize for Email {
   // [...]
}

#[derive(Bynar, Deserialize)]
struct CreateUserRequest {
	#[bynar(fake = "first_name")]
	first_name: Name,
	#[bynar(fake = "last_name")]
	last_name: Name,
	#[bynar(fake = "date_of_birth")]
	date_of_birth: DateOfBirth,
	#[bynar(fake = "email")]
	email: Email
}

fn dummy_create_user_request() -> CreateUserRequest {
	CreateUserRequest::dummy()
}
```

## Getting Help

[todo]

## Project Layout

- [`bynar`](./bynar): Definition of `Bynar`s and core logic.
- [`bynar-derive`](./bynar-derive): Code-generation behind the derive Bynar proc-macros and related attributes.

## Contributing

[todo]

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Tonic by you, shall be licensed as MIT, without any additional terms or conditions.
