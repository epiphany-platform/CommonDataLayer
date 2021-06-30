
```
 Title: Branching Strategy
 Author: Samuel Mohr
 Team: CDL
 Reviewer: CDLTeam
 Created on: 2021-06-30
 Last updated: 2021-06-30
```

# Glossary

- Release Branch - it is a fork originating in the development branch, marking the point in time, that feature set is frozen, and official release is made.
- Development Branch - it is the main, active branch to which all code that is currently worked on is merged. Only developer releases (a.k.a release candidates) can be done from main branch
- MAJOR,MINOR,PATCH - semantic versioning - its a triplet representing 3 main categories of versions major release, minor release, and patches. Decreases in importance from the left to the right. Additionally, a bump in a specific category resets everything in categories to the right to 0.
- Release - usually a build of either release branch or development branch.

# Branching Strategy

## Current Status

When originally developing a strategy for releases, we originally decided to maintain two branches: develop, for all short-term development of features, and main, for stable release of versions, where new versions containing collections of features would be single commits to the release branch, tagged with version numbers.

However, the past few months have led to the team cutting out the middle man and tagging commits in the develop branch, releasing development releases without the need for the main branch. The benefits of having the extra copy were not justified by the extra work of maintaining a separate branch and syncing it with development. Additionally, this didn't solve the issue of how to add bug fixes to already released versions. Though unwise to change existing code, patches made that increment the patch segment of a release's semantic version are a way to fix incorrect behavior of a release without breaking API's or otherwise adding new features. This is not structured into the old approach.

## New Approach

If we consider a release to be a collection of features added since a prior release, our goal in producing releases, is to provide releases using the GitHub releases feature that are accessible to end users, as well as maintaining a means by which we can conveniently and correctly provided patches for said releases.

The proposed solution is a combination of the two most popular means for making releases: making and maintaining release branches, and tagging commits in those branches.

When a release is planned, a branch should be created (sprouted from the common development branch) and named based on the major and minor versions only for the release family (e.g. v1.3).
First, all the features should be completed and then merged into develop. Develop should then be frozen and prepared for next release. A release branch should be then created from the tip of the freeze (if any last-minute fixes are applied).

After release branch is created, all further development will proceed on development branch in the meantime. That means if release deadline will be missed by a developer, freeze does not have to wait for the feature development to release. That also means that no new features can be commited to the released branch, only patches and/or bugfixes.
Release

### Bugfixes

When bug fixes are required to fix incorrect behavior in a release. However, the incorrect behavior can also be present on the develop branch. If the code exists in develop, then a bug fix change PR should be made, tested, and squashed and merged to develop. The squashed commit should then be cherry-picked to all relevant and supported branches, resulting in a bump of version's PATCH number.

If the issue only affects the release code and no longer affects develop code, or the patch is not feasible for cherry-picking (refactored code, too many changes in history etc.) the bugfix PR can be squashed and merged directly to the version branch without needing to cherry-pick from develop.

If the issue affects both development branch and supported release branch(es), but the fix is not feasible to be cherry-picked due to large amount of non-forwardable changes, then fix should be done for each release branch separately from develop, and, if appliable, also introduced to develop.

### Version Branch Lifetime

Release branches have to be supported long term (term will be defined per-case). For example, if we support any release for 6 months, that branch must be kept open for any potential bug fixes that need to be added. When the version is no longer supported, the branch can be marked as obsolete and no further updates to this branch are unlikely to happen. Release branches should not be deleted, overwritten nor squashed.

A set time should be decided for support for all versions generally, though 6 months is a good placeholder for the time being, although, prolonged support may be requested, and therefore it may be necessary to facilitate it.

## TL;DR
#### Patch versioning:
- patch will result in bump of version's PATCH number on affected release branch
- patch will not result in bump in development (release candidates),

#### Patch application:
- if development branch is affected, patch should be fixed and changes should be pushed to development branch
- if supported release branch is also affected by proposed patch... (developer's discretion)
--- ... and fix is easily applicable, it should be cherry-picked from develop to release branch
--- ... but resulting fix is not easy to forward to release branch (i.e. missing refactor, changes in history, conflicting features, etc), fix have to be crafted and applied manually for each branches

#### Tagging of the branch should be done as follows:
- tip of the release branch will be tagged by its (MAJOR,MINOR) tag
- each patch will introduce a build tagged with (MAJOR.MINOR.PATCH), additiona
- development branch will carry release candidate builds (RC), tagged as (MAJOR.MINOR.0-rcXX) where XX is a sequential, increment only value, and (MAJOR.MINOR) are taken from upcoming release

## Out of Scope

This document does not cover methods by which features can be selected to make up new versions, only how to release those versions and how to patch them post-release.
