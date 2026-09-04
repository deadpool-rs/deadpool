local config = std.parseJson(std.extVar("config"));
local rust_version = std.extVar("rust_version");
local crate = std.extVar("crate");

// Pinned action references, keyed by the action name as it is written in a
// `uses:` step. Updating an action is just a matter of changing the
// corresponding line here. The `version` is emitted as a trailing comment on
// the generated `uses:` line (see gen-ci.sh) so that tools like Dependabot can
// track the pinned version.
local actions = {
  "actions/checkout": { sha: "3d3c42e5aac5ba805825da76410c181273ba90b1", version: "v7.0.1" },
  "dtolnay/rust-toolchain": { sha: "6c977a6ca4077a0ceb28ffbe03f59d46e9ac8772", version: "v1" },
  "dcarbone/install-jq-action": { sha: "4fcb5062d7ce9bc4382d1a352d19ba3ba2c317c1", version: "v4.0.1" },
  "dcarbone/install-yq-action": { sha: "4075b4dca348d74bd83f2bf82d30f25d7c54539b", version: "v1.3.1" },
};

// Pin a `uses:` step to the SHA above and tag on `_version`, which the YAML
// generation pass turns into a trailing line comment (see gen-ci.sh). Comments
// written in a crate's `ci.config.yml` are lost when the config is converted to
// JSON, so the map above is the only place a revision may be spelled out --
// naming an action that is not listed there is an error rather than something
// that silently ends up unpinned in a workflow.
local pinAction(name) =
  if std.objectHas(actions, name) then {
    uses: name + "@" + actions[name].sha,
    _version: actions[name].version,
  } else
    error "unpinned action `%s`: add it to the `actions` map in ci.jsonnet" % name;

// Pin every `uses:` step in the document, both the ones defined below and the
// ones coming from a crate's `ci.config.yml`.
local pinActions(node) =
  if std.isArray(node) then
    [pinActions(item) for item in node]
  else if std.isObject(node) then
    { [k]: pinActions(node[k]) for k in std.objectFields(node) }
    + (if std.objectHas(node, "uses") then pinAction(node.uses) else {})
  else
    node;

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

pinActions({
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
        {
          uses: "actions/checkout",
          with: { "persist-credentials": false },
        },
        {
          uses: "dtolnay/rust-toolchain",
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
          uses: "actions/checkout",
          with: { "persist-credentials": false },
        },
        {
          uses: "dtolnay/rust-toolchain",
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
        {
          uses: "actions/checkout",
          with: { "persist-credentials": false },
        },
        {
          uses: "dtolnay/rust-toolchain",
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
          uses: "actions/checkout",
          with: { "persist-credentials": false },
        },
        {
          uses: "dtolnay/rust-toolchain",
          with: {
            toolchain: "nightly",
            components: "rustc,rust-std,cargo",
          }
        },
        {
          uses: "dtolnay/rust-toolchain",
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
          uses: "actions/checkout",
          with: { "persist-credentials": false },
        },
        {
          uses: "dtolnay/rust-toolchain",
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
        {
          uses: "actions/checkout",
          with: { "persist-credentials": false },
        },
        {
          uses: "dtolnay/rust-toolchain",
          with: {
            toolchain: "stable",
            components: "rustc,rust-std,cargo",
          }
        },
        { uses: "dcarbone/install-jq-action" },
        { uses: "dcarbone/install-yq-action" },
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
          uses: "actions/checkout",
          with: { "persist-credentials": false },
        },
        {
          uses: "dtolnay/rust-toolchain",
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
})
