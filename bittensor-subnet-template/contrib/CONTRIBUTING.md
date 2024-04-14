# Contributing to Bittensor Subnet Development

The following is a set of guidelines for contributing to the Bittensor ecosystem. These are **HIGHLY RECOMMENDED** guidelines, but not hard-and-fast rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

## Table Of Contents
1. [How Can I Contribute?](#how-can-i-contribute)
   1. [Communication Channels](#communication-channels)
   1. [Code Contribution General Guideline](#code-contribution-general-guidelines)
   1. [Pull Request Philosophy](#pull-request-philosophy)
   1. [Pull Request Process](#pull-request-process)
   1. [Addressing Feedback](#addressing-feedback)
   1. [Squashing Commits](#squashing-commits)
   1. [Refactoring](#refactoring)
   1. [Peer Review](#peer-review)
 1. [Suggesting Features](#suggesting-enhancements-and-features)


## How Can I Contribute?
TODO(developer): Define your desired contribution procedure.

## Communication Channels
TODO(developer): Place your communication channels here

> Please follow the Bittensor Subnet [style guide](./STYLE.md) regardless of your contribution type. 

Here is a high-level summary:
- Code consistency is crucial; adhere to established programming language conventions.
- Use `black` to format your Python code; it ensures readability and consistency.
- Write concise Git commit messages; summarize changes in ~50 characters.
- Follow these six commit rules:
  - Atomic Commits: Focus on one task or fix per commit.
  - Subject and Body Separation: Use a blank line to separate the subject from the body.
  - Subject Line Length: Keep it under 50 characters for readability.
  - Imperative Mood: Write subject line as if giving a command or instruction.
  - Body Text Width: Wrap text manually at 72 characters.
  - Body Content: Explain what changed and why, not how.
- Make use of your commit messages to simplify project understanding and maintenance.

> For clear examples of each of the commit rules, see the style guide's [rules](./STYLE.md#the-six-rules-of-a-great-commit) section.

### Code Contribution General Guidelines

> Review the Bittensor Subnet [style guide](./STYLE.md) and [development workflow](./DEVELOPMENT_WORKFLOW.md) before contributing. 


#### Pull Request Philosophy

Patchsets and enhancements should always be focused. A pull request could add a feature, fix a bug, or refactor code, but it should not contain a mixture of these. Please also avoid 'super' pull requests which attempt to do too much, are overly large, or overly complex as this makes review difficult. 

Specifically, pull requests must adhere to the following criteria:
- Contain fewer than 50 files. PRs with more than 50 files will be closed.
- If a PR introduces a new feature, it *must* include corresponding tests.
- Other PRs (bug fixes, refactoring, etc.) should ideally also have tests, as they provide proof of concept and prevent regression.
- Categorize your PR properly by using GitHub labels. This aids in the review process by informing reviewers about the type of change at a glance.
- Make sure your code includes adequate comments. These should explain why certain decisions were made and how your changes work.
- If your changes are extensive, consider breaking your PR into smaller, related PRs. This makes your contributions easier to understand and review.
- Be active in the discussion about your PR. Respond promptly to comments and questions to help reviewers understand your changes and speed up the acceptance process.

Generally, all pull requests must:

  - Have a clear use case, fix a demonstrable bug or serve the greater good of the project (e.g. refactoring for modularisation).
  - Be well peer-reviewed.
  - Follow code style guidelines.
  - Not break the existing test suite.
  - Where bugs are fixed, where possible, there should be unit tests demonstrating the bug and also proving the fix.
  - Change relevant comments and documentation when behaviour of code changes.

#### Pull Request Process

Please follow these steps to have your contribution considered by the maintainers:

*Before* creating the PR:
1. Read the [development workflow](./DEVELOPMENT_WORKFLOW.md) defined for this repository to understand our workflow.
2. Ensure your PR meets the criteria stated in the 'Pull Request Philosophy' section.
3. Include relevant tests for any fixed bugs or new features as stated in the [testing guide](./TESTING.md).
4. Ensure your commit messages are clear and concise. Include the issue number if applicable.
5. If you have multiple commits, rebase them into a single commit using `git rebase -i`.
6. Explain what your changes do and why you think they should be merged in the PR description consistent with the [style guide](./STYLE.md).

*After* creating the PR:
1. Verify that all [status checks](https://help.github.com/articles/about-status-checks/) are passing after you submit your pull request. 
2. Label your PR using GitHub's labeling feature. The labels help categorize the PR and streamline the review process.
3. Document your code with comments that provide a clear understanding of your changes. Explain any non-obvious parts of your code or design decisions you've made.
4. If your PR has extensive changes, consider splitting it into smaller, related PRs. This reduces the cognitive load on the reviewers and speeds up the review process.

Please be responsive and participate in the discussion on your PR! This aids in clarifying any confusion or concerns and leads to quicker resolution and merging of your PR.

> Note: If your changes are not ready for merge but you want feedback, create a draft pull request.

Following these criteria will aid in quicker review and potential merging of your PR.
While the prerequisites above must be satisfied prior to having your pull request reviewed, the reviewer(s) may ask you to complete additional design work, tests, or other changes before your pull request can be ultimately accepted.

When you are ready to submit your changes, create a pull request:

> **Always** follow the [style guide](./STYLE.md) and [development workflow](./DEVELOPMENT_WORKFLOW.md) before submitting pull requests.

After you submit a pull request, it will be reviewed by the maintainers. They may ask you to make changes. Please respond to any comments and push your changes as a new commit.

> Note: Be sure to merge the latest from "upstream" before making a pull request:

```bash
git remote add upstream https://github.com/opentensor/bittensor.git # TODO(developer): replace with your repo URL
git fetch upstream
git merge upstream/<your-branch-name>
git push origin <your-branch-name>
```

#### Addressing Feedback

After submitting your pull request, expect comments and reviews from other contributors. You can add more commits to your pull request by committing them locally and pushing to your fork.

You are expected to reply to any review comments before your pull request is merged. You may update the code or reject the feedback if you do not agree with it, but you should express so in a reply. If there is outstanding feedback and you are not actively working on it, your pull request may be closed.

#### Squashing Commits

If your pull request contains fixup commits (commits that change the same line of code repeatedly) or too fine-grained commits, you may be asked to [squash](https://git-scm.com/docs/git-rebase#_interactive_mode) your commits before it will be reviewed. The basic squashing workflow is shown below.

    git checkout your_branch_name
    git rebase -i HEAD~n
    # n is normally the number of commits in the pull request.
    # Set commits (except the one in the first line) from 'pick' to 'squash', save and quit.
    # On the next screen, edit/refine commit messages.
    # Save and quit.
    git push -f # (force push to GitHub)

Please update the resulting commit message, if needed. It should read as a coherent message. In most cases, this means not just listing the interim commits.

If your change contains a merge commit, the above workflow may not work and you will need to remove the merge commit first. See the next section for details on how to rebase.

Please refrain from creating several pull requests for the same change. Use the pull request that is already open (or was created earlier) to amend changes. This preserves the discussion and review that happened earlier for the respective change set.

The length of time required for peer review is unpredictable and will vary from pull request to pull request.

#### Refactoring

Refactoring is a necessary part of any software project's evolution. The following guidelines cover refactoring pull requests for the project.

There are three categories of refactoring: code-only moves, code style fixes, and code refactoring. In general, refactoring pull requests should not mix these three kinds of activities in order to make refactoring pull requests easy to review and uncontroversial. In all cases, refactoring PRs must not change the behaviour of code within the pull request (bugs must be preserved as is).

Project maintainers aim for a quick turnaround on refactoring pull requests, so where possible keep them short, uncomplex and easy to verify.

Pull requests that refactor the code should not be made by new contributors. It requires a certain level of experience to know where the code belongs to and to understand the full ramification (including rebase effort of open pull requests). Trivial pull requests or pull requests that refactor the code with no clear benefits may be immediately closed by the maintainers to reduce unnecessary workload on reviewing.

#### Peer Review

Anyone may participate in peer review which is expressed by comments in the pull request. Typically reviewers will review the code for obvious errors, as well as test out the patch set and opine on the technical merits of the patch. Project maintainers take into account the peer review when determining if there is consensus to merge a pull request (remember that discussions may have taken place elsewhere, not just on GitHub). The following language is used within pull-request comments:

- ACK means "I have tested the code and I agree it should be merged";
- NACK means "I disagree this should be merged", and must be accompanied by sound technical justification. NACKs without accompanying reasoning may be disregarded;
- utACK means "I have not tested the code, but I have reviewed it and it looks OK, I agree it can be merged";
- Concept ACK means "I agree in the general principle of this pull request";
- Nit refers to trivial, often non-blocking issues.

Reviewers should include the commit(s) they have reviewed in their comments. This can be done by copying the commit SHA1 hash.

A pull request that changes consensus-critical code is considerably more involved than a pull request that adds a feature to the wallet, for example. Such patches must be reviewed and thoroughly tested by several reviewers who are knowledgeable about the changed subsystems. Where new features are proposed, it is helpful for reviewers to try out the patch set on a test network and indicate that they have done so in their review. Project maintainers will take this into consideration when merging changes.

For a more detailed description of the review process, see the [Code Review Guidelines](CODE_REVIEW_DOCS.md).

> **Note:** If you find a **Closed** issue that seems like it is the same thing that you're experiencing, open a new issue and include a link to the original issue in the body of your new one.

#### How Do I Submit A (Good) Bug Report?

Please track bugs as GitHub issues.

Explain the problem and include additional details to help maintainers reproduce the problem:

* **Use a clear and descriptive title** for the issue to identify the problem.
* **Describe the exact steps which reproduce the problem** in as many details as possible. For example, start by explaining how you started the application, e.g. which command exactly you used in the terminal, or how you started Bittensor otherwise. When listing steps, **don't just say what you did, but explain how you did it**. For example, if you ran with a set of custom configs, explain if you used a config file or command line arguments. 
* **Provide specific examples to demonstrate the steps**. Include links to files or GitHub projects, or copy/pasteable snippets, which you use in those examples. If you're providing snippets in the issue, use [Markdown code blocks](https://help.github.com/articles/markdown-basics/#multiple-lines).
* **Describe the behavior you observed after following the steps** and point out what exactly is the problem with that behavior.
* **Explain which behavior you expected to see instead and why.**
* **Include screenshots and animated GIFs** which show you following the described steps and clearly demonstrate the problem. You can use [this tool](https://www.cockos.com/licecap/) to record GIFs on macOS and Windows, and [this tool](https://github.com/colinkeenan/silentcast) or [this tool](https://github.com/GNOME/byzanz) on Linux.
* **If you're reporting that Bittensor crashed**, include a crash report with a stack trace from the operating system. On macOS, the crash report will be available in `Console.app` under "Diagnostic and usage information" > "User diagnostic reports". Include the crash report in the issue in a [code block](https://help.github.com/articles/markdown-basics/#multiple-lines), a [file attachment](https://help.github.com/articles/file-attachments-on-issues-and-pull-requests/), or put it in a [gist](https://gist.github.com/) and provide link to that gist.
* **If the problem is related to performance or memory**, include a CPU profile capture with your report, if you're using a GPU then include a GPU profile capture as well. Look into the [PyTorch Profiler](https://pytorch.org/tutorials/recipes/recipes/profiler_recipe.html) to look at memory usage of your model.
* **If the problem wasn't triggered by a specific action**, describe what you were doing before the problem happened and share more information using the guidelines below.

Provide more context by answering these questions:

* **Did the problem start happening recently** (e.g. after updating to a new version) or was this always a problem?
* If the problem started happening recently, **can you reproduce the problem in an older version of Bittensor?** 
* **Can you reliably reproduce the issue?** If not, provide details about how often the problem happens and under which conditions it normally happens.

Include details about your configuration and environment:

* **Which version of Bittensor Subnet are you using?**
* **What commit hash are you on?** You can get the exact commit hash by checking `git log` and pasting the full commit hash.
* **What's the name and version of the OS you're using**?
* **Are you running Bittensor Subnet in a virtual machine?** If so, which VM software are you using and which operating systems and versions are used for the host and the guest?
* **Are you running Bittensor Subnet in a dockerized container?** If so, have you made sure that your docker container contains your latest changes and is up to date with Master branch?

### Suggesting Enhancements and Features

This section guides you through submitting an enhancement suggestion, including completely new features and minor improvements to existing functionality. Following these guidelines helps maintainers and the community understand your suggestion :pencil: and find related suggestions :mag_right:.

When you are creating an enhancement suggestion, please [include as many details as possible](#how-do-i-submit-a-good-enhancement-suggestion). Fill in [the template](https://bit.ly/atom-behavior-pr), including the steps that you imagine you would take if the feature you're requesting existed.

#### Before Submitting An Enhancement Suggestion

* **Check the [debugging guide](./DEBUGGING.md).** for tips — you might discover that the enhancement is already available. Most importantly, check if you're using the latest version of the project first.

#### How Submit A (Good) Feature Suggestion

* **Use a clear and descriptive title** for the issue to identify the problem.
* **Provide a step-by-step description of the suggested enhancement** in as many details as possible.
* **Provide specific examples to demonstrate the steps**. Include copy/pasteable snippets which you use in those examples, as [Markdown code blocks](https://help.github.com/articles/markdown-basics/#multiple-lines).
* **Describe the current behavior** and **explain which behavior you expected to see instead** and why.
* **Include screenshots and animated GIFs** which help you demonstrate the steps or point out the part of the project which the suggestion is related to. You can use [this tool](https://www.cockos.com/licecap/) to record GIFs on macOS and Windows, and [this tool](https://github.com/colinkeenan/silentcast) or [this tool](https://github.com/GNOME/byzanz) on Linux.
* **Explain why this enhancement would be useful** to most users.
* **List some other text editors or applications where this enhancement exists.**
* **Specify the name and version of the OS you're using.**

Thank you for considering contributing to Bittensor! Any help is greatly appreciated along this journey to incentivize open and permissionless intelligence.
