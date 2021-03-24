CDL VERSIONING
======

## Versioning scheme
- CDL is versioned with [semver](https://semver.org/) (MAJOR.MINOR.PATCH)
- Major releases may break your product. Look for changelog/release notes to see if you need to update your code in order for it to work properly with cdl. 
- Minor releases will add new features without breaking any functionality.
- Patch releases should be treated as hofixes. They will not require any changes to your deployment configuration other then version number change. They may introduce new config settings which will be optional.
- Major and minor releases might require changes to deployment configuration. 
## Release
- On each release repository state will be marked with a tag and docker images will be build.
- Standard(non-tagged) releases will be made only from main branch.
- Tagged releases may be created from different branch, unless stated otherwise we do not guarantee theirs stability.
## Breaking changes
By breaking change we mean a change in public api, that is not backward-compatible. Public api defines how clients can query CDL to do some tasks on it. Internal service communication protocol changes are not considered a public api.
## Internal components changes
Once CDL hits 1.0.0 we will support clients with custom versions of CDL components. Clients will be able to write their own repositories(e.g in case no repository for their storage technology exists).

However we can't guarantee that each CDL release will work correctly with custom components. Adding new features to document repository would require custom document repositories to also include those new features. This means that for deployments with custom CDL components each version change can potentially break deployed system. Because of that each component type will have their own version number(different from CDL version). You'll have to look for version changes for each custom component type to determine if you can update CDL safely.
## Release schedule
There is no strict release schedule at this point. New versions will be released when there is a need for them(e.g. new feature implemented).
