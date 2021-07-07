# Front Matter

```
Title           : CDL documentation generation
Author(s)       : Łukasz Biel
Team            : CommonDataLayer
Last updated    : 2021-07-01
Version         : 1.0.0
```

# Glossary

* `CDL` - Common Data Layer
* `QR` - Query Router
* `xtask` - cargo job; similar to how you may define run targets in npm, rake, etc.
* `PoC` - proof of concept
* `Swagger` - Swagger is a tool for generating client and server code from OpenAPI specification. It supports most major languages.

# Preface

The necessity of creating this RFC was sparked by amount of manual work needed when documenting API in `QR`.
Some topics I'm exploring here can be stretched to other parts of CDL documentation, and I'm taking my liberty to include
these parts as well. Thus, reader should mind, that result of this document may span across larger group of components than
`QR` alone.

# Current state

`QR` API is documented manually via [OpenApi][OpenApi] spec file. At the moment of writing this rfc it's outdated.  
`Configuration` is documented manually via [toml][configuration-docs] files compliant with [rust config crate][config-rs].  
`gRPC` configuration is hand-written and [tonic][tonic-rs] generates rust files with code needed for client and server implementation; proto files are located in [rpc crate][cdl-rpc-rs].  
`CDL Input Message` format is documented by hand in [docs][cdl-input-message]. At the moment of writing this rfc it's outdated.  
`Notification Message` format is not documented to my knowing.  
`graphQL` provides documentation via its endpoint and it's built into library we use. It may be wort considering to provide static graphQL spec in repository.

# Problem to solve

Maintaining documentation for formats used in communication within or with CDL is cumbersome.
We need to research options of automating the process and validating its results.

# Possible solutions

## Generate rust code from specification files:

There are options for generating rust code from proto (already in place) and OpenAPI.

For generating structures needed for our internal and external communication, via Kafka and RabbitMQ, we could use [schemafy crate][schemafy-rs].
It most likely needs to be done via `xtask`, similar to how we are generating things using [tonic][tonic-rs].

As for OpenAPI, it would require us to switch away from using [warp][warp-rs] in favour of [hyper][hyper-rs]. Swagger can generate new crate in our repo, 
that `QR` would use as a base.

For graphQL api - we would need to provide a `xtask` which pulls spec from running api instance. There's no easy way to generate rust code from existing api spec. If we were to stick with this approach - we'd have to write something in-house.

TOML configuration is a bit problematic. There are no known crates that can easily generate rust structs out from Toml config, and our structs
are usually reused. What I mean is that generating code would introduce duplication. It's not necessarily bad, but that's confusing.
We probably would need to produce an in-house solution for that problem.

In general - it requires some significant work from team to switch fully to configs generating rust code and it can't be done easily in all of the places.

## Generate configs from rust

### Compiler pass way:

We can create a binary that walks through our codebase and based on some criteria generates specs based on rust AST. It's doable, although not an easy task.
More can be read searching for feature `rustc_private`, it allows users to use rustc as a library.

### XTask + Lib way:

We can create special traits/methods and use them within custom binaries used only in `xtask` targets. This way compiler should remove unused instances of code in official binaries,
no extra handling would be visible on production side, but `xtask` target would be able to perform actions based on these traits.

We would do nothing for `graphQL`, yet create task that starts an instance and fetches spec.

We would have to work hard on `QR` as warp doesn't necessarily play with such code generation - 
I don't think it's wise to decorate function expressions with macros, and methods themselves don't provide all info out of the box.
We would have to mark arguments of these methods with where does it come from. Some are provided by header, some are caches, some are bodies or query params. 
If we wouldn't compromise speed and latency, I'd propose to review other options for servers in rust,
as I believe they may be easier to integrate with `xtask` approach. Alas, there's a lot of uncertainty, and some of the research definitely requires a PoC.

For configuration, it should be really easy to write these traits from scratch. It's ever possible to use `serde` own traits within our own (for defaults and whatnot).
For JSONs, in form of `CDL Input Message` and `Notification Message`, goes the same. Just, we can use existing solution.

`gRPC` may prove a challenge, more research is to be done, probably PoC. Tonic supports generation one way, 
and I'm not sure how much of an effort would it be to replace generated files with something we'd be able to write manually and generate protos from it.
[tarpc][tarpc-rs] may be worth looking at, as it's purely macro based. We've put some work into it, however I cannot recall any PoC using protobuf format. 
Some context may be found in [this issue][tarpc-rs-161], it appears it's possible. For `xtask` this still would mean custom `.protobuf` gen.

## Mixed solution

It's not a problem to have mixed solution in some places. Important part is nothing has to be done manually in more than one place.

# Author's comment

As generating rust code may be tempting and is quite popular in rest of the industry, I think it may be beneficial to generate specs from rust.
Rust macro system is quite easy to use and this differentiates it from most of other languages. We will avoid compiling / and possibly reviewing / specs.
Some initial discussion happened on this topic, and it seems that team, in majority, agrees with that. Next step would be to PoC this.

# Conclusion

TBD

[OpenApi]: https://swagger.io/specification/
[configuration-docs]: ../configuration/index.md
[cdl-input-message]: ../architecture/data_router.md
[cdl-rpc-rs]: https://github.com/epiphany-platform/CommonDataLayer/tree/develop/crates/rpc/proto
[tonic-rs]: https://github.com/hyperium/tonic
[config-rs]: https://github.com/mehcode/config-rs
[schemafy-rs]: https://docs.rs/schemafy/0.5.2/schemafy/
[warp-rs]: https://github.com/seanmonstar/warp
[hyper-rs]: https://github.com/hyperium/hyper
[tarpc-rs]: https://github.com/google/tarpc
[tarpc-rs-161]: https://github.com/google/tarpc/issues/161
