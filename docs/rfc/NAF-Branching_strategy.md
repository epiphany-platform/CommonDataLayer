
```
 Title: Branching Strategy
 Author: Samuel Mohr
 Team: CDL
 Reviewer: CDLTeam
 Created on: 2021-06-30
 Last updated: 2021-06-30
```

````
=========================================================
shortened version is available at the end of the document
=========================================================

````



# Glossary

- Release Branch - A fork originating from the development branch, marking the point in time, in which a feature set is frozen, and an official release is made.
- Development Branch - The main, active branch to which all code that is currently worked on is merged. Only developer releases (a.k.a. release candidates) can be done from this main branch.
- MAJOR,MINOR,PATCH - The standard notation for semantic versioning. The triplet represents three main categories of versions: major release, minor release, and patches. The importance of the version section decreases from left to right. Additionally, a bump in a specific category resets everything in categories to the right to 0.
- Release - A fixed build of either the release branch or development branch that is then published for public usage.


# Current Status

When originally developing a strategy for releases, we originally decided to maintain two branches: develop, for all short-term development of features, and main, for stable release of versions, where new versions containing collections of features would be single commits to the release branch, tagged with version numbers.

However, the past few months have led to the team cutting out the middle man and tagging commits in the develop branch, releasing development releases without the need for the main branch. The benefits of having the extra copy were not justified by the extra work of maintaining a separate branch and syncing it with development. Additionally, this didn't solve the issue of how to add bug fixes to already released versions. Though unwise to change existing code, patches made that increment the patch segment of a release's semantic version are a way to fix incorrect behavior of a release without breaking API's or otherwise adding new features. This is not structured into the old approach.


# New Approach

If we consider a release to be a collection of features added since a prior release, our goal in producing releases, is to provide releases using the GitHub releases feature that are accessible to end users, as well as maintaining a means by which we can conveniently and correctly provided patches for said releases.

The proposed solution is a combination of the two most popular means for making releases: making and maintaining release branches, and tagging commits in those branches.

When a release is planned, the next branch should be created (sprouted from the common development branch) with name based on the major and minor versions only for the release family (e.g. v1.3).
First, all the features should be completed and then merged into develop. Develop should then be frozen and prepared for next release. A release branch should be then created from the tip of the freeze (if any last-minute fixes are applied).
If a feature is planned but not delivered in time before the freeze, then afterward the freeze not delivered changes should not be committed (only last-minute fixes to existing ones), until release branch is created, main branch is treated as one and it can not get new features. Additionally, a release branch's origin cannot be moved after creation and features cannot be cherry-picked or merged afterward.

After the release branch is created, all further development will proceed on development branch simultaneously. That means, if the release deadline will be missed by a developers, freeze does not have to wait for the feature development to release. That also means that no new features can be committed to the released branch, only patches and/or bug fixes.

## Bugfixes

When bug fixes are required to fix incorrect behavior in a release. However, the incorrect behavior can also be present on the develop branch. If the code exists in develop, then a bug fix change PR should be made, tested, and squashed and merged to develop. The squashed commit should then be cherry-picked to all relevant and supported branches, resulting in a bump of the version's PATCH number.

If the issue only affects the release code and no longer affects develop code, or the patch is not feasible for cherry-picking (refactored code, too many changes in history etc.) the bug fix PR can be squashed and merged directly to the version branch without needing to cherry-pick from develop.

If the issue affects both development branch and supported release branch(es), but the fix is not feasible to be cherry-picked due to large amount of non-forwardable changes, then fix should be done for each release branch separately from develop, and, if appliable, also introduced to develop.

## Version Branch Lifetime

Release branches have to be supported long term (term will be defined per-case). For example, if we support any release for 6 months, that branch must be kept open for any potential bug fixes that need to be added. When the version is no longer supported, the branch can be marked as obsolete and no further updates to this branch are unlikely to happen. Release branches should not be deleted, overwritten nor squashed.

A set time should be decided for support for all versions generally, though 6 months is a good placeholder for the time being, although, prolonged support may be requested, and therefore it may be necessary to facilitate it.

Currently, we are responsible for keeping 3 last releases in support pipeline. That means 3 quarters or 9 months of releases. Additionally, please keep in mind that RC releases are preview branches and do not have to be supported nor fixes for those are a priority.


# TL;DR

This document is describing what is called, `scaled trunk based development` and many materials are freely available online in different sources.
This document however introduces and specifies how tagging and release preparations should look like. Besides that, everything should be 1:1 with official documentation about the process.

### Patch Versioning

- Patches will result in a bump of the current version's PATCH number on the affected release branch.
- Patches will not result in bump in development (release candidates).

### Patch Application

- If the development branch is affected, the patch should be fixed and changes should be pushed to the development branch.
- If the supported release branch is also affected by the proposed patch... (to the developer's discretion):
  - ... and the fix is easily applicable, it should be cherry-picked from develop to release branch.
  - ... but the resulting fix is not easy to forward to the release branch (i.e. missing refactor, changes in history, conflicting features, etc), the fix has to be crafted and applied manually for each branch.

### Branch Tagging

- The tip of the release branch will be tagged by its (MAJOR,MINOR) tag.
- Each patch will additionally introduce a build tagged with (MAJOR.MINOR.PATCH).
- The development branch will carry release candidate builds (RC), tagged as (MAJOR.MINOR.0-rcXX) where XX is a sequential, increment-only value, and (MAJOR.MINOR) are taken from the upcoming release.


# Out of Scope

This document does not cover methods by which features can be selected to make up new versions, only how to release those versions and how to patch them post-release.
