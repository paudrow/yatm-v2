# YATM v2

YATM v2 is a tool for generating test cases from requirements for manual testing.
It works well with putting issues on Github so that community contributors can help with testing.

YATM v2 is a rewrite of the [original YATM tool](https://github.com/audrow/yatm), which was written in Typescript and used for several [ROS 2](https://docs.ros.org/) and [Gazebo](https://gazebosim.org/home) releases by [Open Robotics](https://www.openrobotics.org/) and [Intrinsic AI](https://intrinsic.ai/).

## Features

- Specify requirements in YAML files
- Specify a test case builder to generate test cases from requirements
- CLI interface
- Upload test cases to Github issues (don't worry, it won't make duplicates)
- Preview test cases locally in a markdown file before uploading to Github
- Init a new YATM v2 project from the command line that sets up the config, file structure, and comes with sensible defaults
- Migrate requirements from YATM v1 to v2
- Verification commands to check for valid requirements and test case builder files. This also makes it easier to write code in other languages than Rust to generate requirements.
- CLI tool for getting metrics on test cases on Github
- Several Github utility functions to make common operations easier
- Uses a tag to version of your YATM v2 workspace to make it easy to separate different iterations of your test cases, for uploading and analytics purposes.
- Extensible design so you can build requirements builders.
- An templated file for the Github issue template, which makes it much easier to customize the issue template for your project.

## How to use YATM v2

### Pre-requisites

1. Install Rust

    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

1. Create a Github repository for your project and note the owner and name of the repository. For example, if the URL of your repository is `https://github.com/paudrow/yatm-v2`, then the owner is `paudrow` and the name is `yatm-v2`.

1. Get a personal access token from Github. This token should have read and write access to the following.

    * `Actions` (not 100% sure if this is necessary)
    * `Contents`
    * `Issues`
    * `Pull requests`

    Note that you can give the token permissions to only the repositories you want to use YATM v2 with.


### Getting started

1. Clone this repository and check that the tests pass.

    ```bash
    # from the root of this repository
    cargo test
    ```

1. Install YATM v2 from the root of this repository

    ```bash
    cargo install --path src/yatm_v2
    ```

    > You can uninstall YATM v2 with `cargo uninstall yatm_v2`

1. Initalize a new YATM v2 workspace

    ```bash
    yatm_v2 init --path /path/to/your/workspace
    ```

    This will create a new YATM v2 workspace with a `config.yaml` file and a `requirements` directory.

1. In `config.yaml`, modify the `repo_owner` and `repo_name` to match your Github repository.

1. Make sure that your workspace has access to your personal access token through the `GITHUB_TOKEN` environment variable.

    YATM v2 will look in the `.env` file in the root of your workspace for the `GITHUB_TOKEN` environment variable. This file should look like this:

    ```bash
    # .env
    GITHUB_TOKEN="github_pat_**********************************************************************************"
    ```

    > You shouldn't commit this file to a public repository. It's already in the `.gitignore` file generated with each YATM v2 workspace.

1. Generate a preview of your test cases:

    ```bash
    yatm_v2 github preview
    ```

    This will generate a markdown file with your test cases in the `generated_files` (unless you've set a different path in `config.yaml`).

    If this looks good, you can upload your test cases to Github.

1. Upload your test cases to Github:

    ```bash
    yatm_v2 github upload
    ```

    This will create a new issue on your Github repository with the test cases.

If you've gotten this far, you've successfully set up YATM v2 for your project. You can now start adding requirements and generating test cases.

## Usage

### Terminology

- **Requirement**: A requirement specifies how a system should behave in a given situation. These are in the form of "Actions" that should be taken and "Expected Results" that should occur.

- **Test case**: A test case is a requirement that has been given a specific testing environment, such as on Ubuntu 22.04 and with Cyclone DDS.

> You can think of a requirement as an abstract class and a test case as an instance of that class.

- **Test case builder**: A test case builder is a mapping of requirements to test cases. It specifies which requirements should be included and which permutations of several possible environments should be used.

### Workflow

1. Initialize a new YATM v2 workspace with `yatm_v2 init --path /path/to/your/workspace`.

1. Modify the `config.yaml` file to match your Github repository.

1. Add a `.env` file to the root of your workspace with your Github personal access token.

1. (Optional) Prepare your repository by resetting the labels.

    ```bash
    # type 'yes' when asked to confirm
    yatm_v2 github utils delete-all-labels
    ```

    Then you can create new labels specificed in the `config.yaml` file. There are some default labels that are created with `yatm_v2 init`, but you modify them or add more.

    ```bash
    yatm_v2 github utils create-labels
    ```

1. Add requirements to the `requirements` directory. Optionally, verify that they are valid with `yatm_v2 requirements validate`.

    You can generate a starter requirements file with the following command:
    ```yaml
    yatm_v2 requirements new
    ```

1. Create a test case builder in the `test_cases_builder` directory. Optionally, verify that it is valid with `yatm_v2 test-cases validate`.

    You can generate a starter test case builder file with the following command:
    ```yaml
    yatm_v2 test-cases new
    ```

1. (Optional) Generate a preview of your test cases with `yatm_v2 github preview`.

1. Upload your test cases to Github with `yatm_v2 github upload`.

1. Create a meta issue to direct your users and create a list of links to make it easier to find the test cases.

    ```bash
    yatm_v2 github make-label-links
    ```

    This will give you something like the following in your project's `generated_files` directory:

    ```md
    - [`Build type: Debian`, `Chip set: AMD64`, `DDS: FastDDS`, `OS: Ubuntu Jammy 22.04`](https://github.com/paudrow/test-yatm-v2/issues?q=is:issue+is:open+label:%22Build+type:+Debian%22+label:%22Chip+set:+AMD64%22+label:%22DDS:+FastDDS%22+label:%22OS:+Ubuntu+Jammy+22.04%22)
    - [`Build type: Binary`, `Chip set: AMD64`, `DDS: FastDDS`, `OS: Ubuntu Jammy 22.04`](https://github.com/paudrow/test-yatm-v2/issues?q=is:issue+is:open+label:%22Build+type:+Binary%22+label:%22Chip+set:+AMD64%22+label:%22DDS:+FastDDS%22+label:%22OS:+Ubuntu+Jammy+22.04%22)
    ```

1. (Optional) Get metrics on your test cases.

    ```bash
    yatm_v2 github metrics
    ```

    Which will give you something like the following:

    ```md
    2/18 issues closed: 11.11%
    ```

    You can also get metrics for a specific label.

    ```bash
    yatm_v2 github metrics --label "who: community tested"
    ```

    Which will give you something like the following:

    ```md
    2/3 issues closed: 66.67%
    ```

    This can be a great way to see how well we are doing with testing and to measure the impact of community contributions.

1. If you find that you need to make changes to your requirements or test case builder, you can

    1. Close all of the issues on Github with the current workspace version.

    ```bash
    yatm_v2 github close-all-issues --label "version: <current version>"
    ```

    1. Bump the version of your workspace in `config.yaml`.

    1. Upload your new test cases to Github.

    ```bash
    yatm_v2 github upload
    ```

### Parts of a requirements file

Here is an example of a requirements file:

```yaml
requirements:
- name: name
  description: description
  steps:
  - name: step name
    description: step description
    action:
    - !Describe action
    - !StdIn
      number: 1
      text: echo 'hi'
    - !Url
      name: Google
      url: www.google.com
    - !Image https://placekitten.com/200/300
    expect:
    - !Describe expect
    - !StdOut
      number: 1
      text: hi
    - !StdErr
      number: 1
      text: error
    - !Url
      name: Google
      url: www.google.com
    - !Image https://placekitten.com/200/300
  labels:
  - label
  links:
  - name: Google
    url: www.google.com
```

The main thing to understand is that a requirement has one or more steps. Each step has an action and an expect. The action is what the user should do and the expect is what the system should output.

There are several different types of actions and expects. For example, `!StdIn` is an action that specifies that the user should input something into the system. `!StdOut` is an expect that specifies that the system should output something.

You'll see all of them in the example above.

One thing to note is that `!StdIn`, `!StdOut`, and `!StdErr` all have a `number` field. This is the terminal number for the action or expect. This is useful for when you have multiple actions or expects in a single step.

Also note that `!Image` will only use an image that can be accessed through a URL. You can put images on Github Gists and use the raw URL to use them in your requirements.

### Parts of a test case builder file

Here is an example of a test case builder file:

```yaml
test_cases_builders:
- name: Demo test cases
  description: description
  set:
  - !Include
    all_labels:
    - label
    any_names:
    - name
    negate: false
  - !Exclude
    all_labels: null
    any_names:
    - Demo
    negate: false
  labels:
  - Demo
  permutations:
    Operating System:
    - Ubuntu 22.04
    - Windows 11
    - MacOS 12.0
    RMW:
    - CycloneDDS
    - FastDDS
```

The test case builder has two main parts:

- `set`: This is a set of requirements that should be included in the test cases. In this case, we're including all requirements.

- `permutations`: This is a mapping of labels to a list of possible values. In this case, we're specifying that we want to generate test cases for each permutation of the Operating System and RMW labels.


#### Understanding the `!Include` and `!Exclude` directives

From the above example, we have the following:

```yaml
test_cases_builders:
  ...
  set:
  - !Include
    all_labels:
    - label
    any_names:
    - name
    negate: false
  - !Exclude
    all_labels: null
    any_names:
    - Demo
    negate: false
  ...
```

Both `!Include` and `!Exclude` have the following fields:

- `all_labels`: A list of labels that all requirements must have to be included or excluded. If `null`, then this field is ignored.

    A label will match if it is an exact match of a requirement label. If more than one label is specified, then a requirement must have all of the labels to match.

- `any_names`: A list of names that any requirement must have to be be included or excluded. If `null`, then this field is ignored.

    A name will match if it is a substring of the requirement name. For example, if `any_names` is `['demo', 'foo']`, then a requirement with the name `Demo 1` will match (`demo` matches). Note that this is case insensitive.

    If more than one name is specified, then a requirement must have at least one of the names to match.

- `negate`: A boolean that specifies if the `all_labels` and `any_names` fields should be negated. If `true`, then the `all_labels` and `any_names` fields are negated.

For example, if `!Include` has `all_labels: [A, B]` and `any_names: [C, D]`, then a requirement must have labels `A` and `B` and at least one of the names `C` and `D` to be included.

If `!Include` has `negate: true`, then a requirement must not have labels `A` and `B` and at least one of the names `C` and `D` to be included.

> It's important to note that `!Include` and `!Exclude` are applied in the order they are specified in the `set` field.This means that you can include all requirements and then exclude some of them.

This is a powerful way to build test cases from requirements.

#### Understanding Permutations

In the above example we have the following:

```yaml
  #...
  permutations:
    Operating System:
    - Ubuntu 22.04
    - Windows 11
    - MacOS 12.0
    RMW:
    - CycloneDDS
    - FastDDS
```

This will generate six test cases, one for each permutation of the Operating System and RMW labels.

- Ubuntu 22.04, CycloneDDS
- Ubuntu 22.04, FastDDS
- Windows 11, CycloneDDS
- Windows 11, FastDDS
- MacOS 12.0, CycloneDDS
- MacOS 12.0, FastDDS

For every key (Operating System and RMW), there will be a test case for each value.

