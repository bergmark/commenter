### 0.4.2 (not released)
* `grandfather`: New command to add mentioned packages that ar not maintained to Grandfathered Dependencies.

### 0.4.1
* `add`: Tweak bound messages

## 0.4.0
* Add `package-info` command
* Add restrictive bounds info to bounds failures added by `add` and `add-loop`
* `latest-version` is no longer a separate binary, so no need to install it anymore.

## 0.3.0

* Add release binaries
* `add-loop` command to iterate `check` and `commend add` until there
  is a valid build plan. Pass `--clear` to remove all generated bounds
  before running.
