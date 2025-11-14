# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## `v0.1.4`
### Bug Fixes

- fix thread usage by @mempirate in
https://github.com/chainbound/prometric/pull/39
- don't enable process feature by default by @mempirate in
https://github.com/chainbound/prometric/pull/33

### Features
- support expressions that evalute into a Vec<f64> for buckets by @thedevbirb in
https://github.com/chainbound/prometric/pull/37
- add thread busyness stats by @mempirate in
https://github.com/chainbound/prometric/pull/38
- add collection time metric, more system stats by @mempirate in
https://github.com/chainbound/prometric/pull/33

## `v0.1.3`
### Bug Fixes
- fix CPU usage, no default feature by @mempirate in
  <https://github.com/chainbound/prometric/pull/30>
- default scrape path /metrics by @mempirate in
  <https://github.com/chainbound/prometric/pull/30>
- blocking issue by @mempirate in
  <https://github.com/chainbound/prometric/pull/30>

### Documentation
- fix docs by @mempirate in
  <https://github.com/chainbound/prometric/pull/30>
- document process metrics by @mempirate in
  <https://github.com/chainbound/prometric/pull/30>
- update README doc order by @mempirate in
  <https://github.com/chainbound/prometric/pull/26>
- add exporter docs by @mempirate in
  <https://github.com/chainbound/prometric/pull/26>
- add metric constructor documentation by @mempirate in
  <https://github.com/chainbound/prometric/pull/25>
- more metric types documentation by @mempirate in
  <https://github.com/chainbound/prometric/pull/21>

### Features
- add exporter example by @mempirate in
  <https://github.com/chainbound/prometric/pull/26>
- add HTTP exporter utilities by @mempirate in
  <https://github.com/chainbound/prometric/pull/26>
- don't collect system swap by @mempirate in
  <https://github.com/chainbound/prometric/pull/30>
- add some system metrics by @mempirate in
  <https://github.com/chainbound/prometric/pull/30>
- add default impl by @mempirate in
  <https://github.com/chainbound/prometric/pull/30>
- add process metrics by @mempirate in
  <https://github.com/chainbound/prometric/pull/30>
- add expression support for buckets by @thedevbirb in
  <https://github.com/chainbound/prometric/pull/37>
