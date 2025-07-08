local config = std.parseJson(std.extVar("config"));
local rust_version = std.extVar("rust_version");
local crate = std.extVar("crate");

local config_test = if std.objectHas(config, "test") then config.test else {};

{
  name: crate,
  on: {
    push: {
        branches: [ "main" ],
        tags: [ std.format("%s-v*", crate) ],
        paths: [ std.format("crates/%s/**", crate) ]
    },
    pull_request: {
        branches: [ "main" ],
        paths: [ std.format("crates/%s/**", crate) ]
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
          uses: "actions/checkout@v3"
        },
        {
          uses: "actions-rs/toolchain@v1",
          with: {
            profile: "minimal",
            toolchain: "stable",
            components: "clippy",
          }
        },
        {
          run: "cargo clippy --no-deps --all-features -- -D warnings"
        }
      ]
    },
    rustfmt: {
      name: "rustfmt",
      "runs-on": "ubuntu-latest",
      steps: [
        {
          uses: "actions/checkout@v3",
        },
        {
          uses: "actions-rs/toolchain@v1",
          with: {
            profile: "minimal",
            toolchain: "stable",
            components: "rustfmt",
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

    msrv: {
      name: "MSRV",
      "runs-on": "ubuntu-latest",
      steps: [
        {
          uses: "actions/checkout@v3"
        },
        {
          uses: "actions-rs/toolchain@v1",
          with: {
            profile: "minimal",
            toolchain: "nightly",
          }
        },
        {
          uses: "actions-rs/toolchain@v1",
          with: {
            profile: "minimal",
            toolchain: rust_version,
            override: "true",
          }
        },
        {
          run: "cargo +nightly update"
        },
        {
          run: "cargo +nightly update -Z minimal-versions"
        },
        {
          run: "cargo check --all-features"
        },
      ],
    },

    test: {
      name: "Test",
      "runs-on": "ubuntu-latest",
      services: if std.objectHas(config_test, "services") then config_test.services else {},
      steps: [
        {
          uses: "actions/checkout@v3",
        },
        {
          uses: "actions-rs/toolchain@v1",
          with: {
            profile: "minimal",
            toolchain: "stable",
          }
        },
        {
          run: "cargo test --all-features",
          env: if std.objectHas(config_test, "env") then config_test.env else {},
        },
      ],
    },

    # check-features:
    #   name: Check features
    #   runs-on: ubuntu-latest
    #   steps:
    #     - uses: actions/checkout@v3
    #     - uses: actions-rs/toolchain@v1
    #       with:
    #         profile: minimal
    #         toolchain: stable
    #     - uses: dcarbone/install-jq-action@v3
    #     - run: .github/check-features/compare-features.sh deadpool-postgres tokio_postgres

    ############
    # Building #
    ############

    rustdoc: {
      name: "Doc",
      "runs-on": "ubuntu-latest",
      steps: [
        {
          uses: "actions/checkout@v3",
        },
        {
          uses: "actions-rs/toolchain@v1",
          with: {
            profile: "minimal",
            toolchain: "stable",
          }
        },
        {
          run: "cargo doc --no-deps --all-features",
        }
      ],
    },
  }
}
