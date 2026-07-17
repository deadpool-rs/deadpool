local config = std.parseJson(std.extVar("config"));
local rust_version = std.extVar("rust_version");
local crate = std.extVar("crate");

// Pinned action references. Updating an action is just a matter of
// changing the corresponding line here. The `version` is emitted as a
// trailing comment on the generated `uses:` line (see gen-ci.sh) so that
// tools like Dependabot can track the pinned version.
local actions = {
  checkout: { ref: "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0", version: "v7.0.0" },
  rust_toolchain: { ref: "dtolnay/rust-toolchain@e97e2d8cc328f1b50210efc529dca0028893a2d9", version: "v1" },
  install_jq: { ref: "dcarbone/install-jq-action@4fcb5062d7ce9bc4382d1a352d19ba3ba2c317c1", version: "v4.0.1" },
  install_yq: { ref: "dcarbone/install-yq-action@4075b4dca348d74bd83f2bf82d30f25d7c54539b", version: "v1.3.1" },
};

// Build a `uses:` step. `_version` is tagged onto the step so the YAML
// generation pass can turn it into a trailing line comment.
local step(action, extra={}) = { uses: action.ref, _version: action.version } + extra;

local getPathOrDefault(obj, path, default) =
  if std.length(path) == 0 then
    obj
  else if std.type(obj) != "object" then
    default
  else if std.objectHas(obj, path[0]) then
    getPathOrDefault(obj[path[0]], path[1:], default)
  else
    default;

local getConfig(path, default) =
  getPathOrDefault(config, std.split(path, "."), default);

local backend = getConfig("backend", null);
local features_own = getConfig("features.own", null);
// features.optional_dependencies is a list of features which are
// modelled by optional dependencies.
local features_required = getConfig("features.required", null);
local features =
  if features_own != null || features_required != null then
    (if features_own != null then features_own else []) +
    (if features_required != null then features_required else [])
  else
    null;
local check_features = getConfig("check.features", features);
local check_extra_steps = getConfig("check.extra_steps", []);
local test_features = getConfig("test.features", features);
local test_services = getConfig("test.services", {});
local test_env = getConfig("test.env", {});
local jobs = getConfig("jobs", {});

local genFeaturesFlag(features) =
  if features != null then
    if std.length(features) > 0 then
      " --features " + std.join(",", features)
    else
      ""
  else
    " --all-features";

{
  name: crate,
  permissions: {},
  on: {
    push: {
        branches: [ "main" ],
        tags: [ std.format("%s-v*", crate) ],
        paths: [ std.format("crates/%s/**", crate), std.format(".github/workflows/%s.yml", crate) ]
    },
    pull_request: {
        branches: [ "main" ],
        paths: [ std.format("crates/%s/**", crate), std.format(".github/workflows/%s.yml", crate) ]
    }
  },
  env: {
    CARGO_NET_RETRY: 10,
    RUST_BACKTRACE: 1
  },
  defaults: {
    run: {
      "working-directory": std.format("./crates/%s", crate),
    }
  },
  jobs: {

    ##########################
    # Linting and formatting #
    ##########################

    clippy: {
      name: "Clippy",
      "runs-on": "ubuntu-latest",
      steps: [
        step(actions.checkout, {
          with: { "persist-credentials": false },
        }),
        step(actions.rust_toolchain, {
          with: {
            toolchain: "stable",
            components: "rustc,rust-std,cargo,clippy",
          }
        }),
        {
          run: "cargo clippy --no-deps" + genFeaturesFlag(features) + " -- -D warnings"
        }
      ]
    },
    rustfmt: {
      name: "rustfmt",
      "runs-on": "ubuntu-latest",
      steps: [
        step(actions.checkout, {
          with: { "persist-credentials": false },
        }),
        step(actions.rust_toolchain, {
          with: {
            toolchain: "stable",
            components: "rustc,rust-std,cargo,rustfmt",
          }
        }),
        {
          run: "cargo fmt --check",
        },
      ],
    },

    ###########
    # Testing #
    ###########

    # FIXME The check integration job should be enabled for all crates with a backend
    [if check_features != null then "check-integration"]: {
      name: "Check integration",
      strategy: {
        "fail-fast": false,
        matrix: {
          feature: check_features,
          os: ["ubuntu-latest", "windows-2025"],
        }
      },
      "runs-on": "${{ matrix.os }}",
      steps: [
        step(actions.checkout, {
          with: { "persist-credentials": false },
        }),
        step(actions.rust_toolchain, {
          with: {
            toolchain: "stable",
            components: "rustc,rust-std,cargo",
          }
        }),
      ] + check_extra_steps + [
        # We don't use `--no-default-features` here as integration crates don't
        # work with it at all.
        {
          run: "cargo check --features ${{ matrix.feature }}"
        }
      ]
    },

    msrv: {
      name: "MSRV",
      "runs-on": "ubuntu-latest",
      steps: [
        step(actions.checkout, {
          with: { "persist-credentials": false },
        }),
        step(actions.rust_toolchain, {
          with: {
            toolchain: "nightly",
            components: "rustc,rust-std,cargo",
          }
        }),
        step(actions.rust_toolchain, {
          with: {
            toolchain: rust_version,
            components: "rustc,rust-std,cargo",
          }
        }),
        {
          run: "../../tools/cargo-update-minimal-versions.sh " + rust_version,
        },
        {
          run: "cargo check" + genFeaturesFlag(features)
        },
      ],
    },

    test: {
      name: "Test",
      "runs-on": "ubuntu-latest",
      services: test_services,
      steps: [
        step(actions.checkout, {
          with: { "persist-credentials": false },
        }),
        step(actions.rust_toolchain, {
          with: {
            toolchain: "stable",
            components: "rustc,rust-std,cargo",
          }
        }),
        {
          run: "cargo test" + genFeaturesFlag(test_features),
          env: test_env,
        },
      ],
    },

    [if backend != null then "check-reexported-features"]: {
      name: "Check re-exported features",
      "runs-on": "ubuntu-latest",
      steps: [
        step(actions.checkout, {
          with: { "persist-credentials": false },
        }),
        step(actions.rust_toolchain, {
          with: {
            toolchain: "stable",
            components: "rustc,rust-std,cargo",
          }
        }),
        step(actions.install_jq),
        step(actions.install_yq),
        { run: "../../tools/check-reexported-features.sh" },
      ]
    },

    ############
    # Building #
    ############

    rustdoc: {
      name: "Doc",
      "runs-on": "ubuntu-latest",
      steps: [
        step(actions.checkout, {
          with: { "persist-credentials": false },
        }),
        step(actions.rust_toolchain, {
          with: {
            toolchain: "stable",
            components: "rustc,rust-std,cargo",
          }
        }),
        {
          run: "cargo doc --no-deps" + genFeaturesFlag(features),
        }
      ],
    },
  }
  + jobs
}
