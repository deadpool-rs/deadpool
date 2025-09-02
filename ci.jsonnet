local config = std.parseJson(std.extVar("config"));
local rust_version = std.extVar("rust_version");
local crate = std.extVar("crate");

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
        {
          uses: "actions/checkout@v5"
        },
        {
          uses: "dtolnay/rust-toolchain@v1",
          with: {
            toolchain: "stable",
            components: "rustc,rust-std,cargo,clippy",
          }
        },
        {
          run: "cargo clippy --no-deps" + genFeaturesFlag(features) + " -- -D warnings"
        }
      ]
    },
    rustfmt: {
      name: "rustfmt",
      "runs-on": "ubuntu-latest",
      steps: [
        {
          uses: "actions/checkout@v5",
        },
        {
          uses: "dtolnay/rust-toolchain@v1",
          with: {
            toolchain: "stable",
            components: "rustc,rust-std,cargo,rustfmt",
          }
        },
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
        { uses: "actions/checkout@v5" },
        {
          uses: "dtolnay/rust-toolchain@v1",
          with: {
            toolchain: "stable",
            components: "rustc,rust-std,cargo",
          }
        },
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
        {
          uses: "actions/checkout@v5"
        },
        {
          uses: "dtolnay/rust-toolchain@v1",
          with: {
            toolchain: "nightly",
            components: "rustc,rust-std,cargo",
          }
        },
        {
          uses: "dtolnay/rust-toolchain@v1",
          with: {
            toolchain: rust_version,
            components: "rustc,rust-std,cargo",
          }
        },
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
        {
          uses: "actions/checkout@v5",
        },
        {
          uses: "dtolnay/rust-toolchain@v1",
          with: {
            toolchain: "stable",
            components: "rustc,rust-std,cargo",
          }
        },
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
        { uses: "actions/checkout@v5" },
        {
          uses: "dtolnay/rust-toolchain@v1",
          with: {
            toolchain: "stable",
            components: "rustc,rust-std,cargo",
          }
        },
        { uses: "dcarbone/install-jq-action@v3" },
        { uses: "dcarbone/install-yq-action@v1" },
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
        {
          uses: "actions/checkout@v5",
        },
        {
          uses: "dtolnay/rust-toolchain@v1",
          with: {
            toolchain: "stable",
            components: "rustc,rust-std,cargo",
          }
        },
        {
          run: "cargo doc --no-deps" + genFeaturesFlag(features),
        }
      ],
    },
  }
  + jobs
}
