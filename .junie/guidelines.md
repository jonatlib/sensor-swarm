# STM32 based sensor swarm peroject

## Rust

We are using modern Rust where mod.rs is not needed and you can directly write same filename and same directory name as sub-module.

## Project standards

We are writing documented and tested code:
Documentation:
  - All methods, structs, modules have a documentation string
  - If possible we are writing in-line tests (using `#[cfg(tests)]`). Beware we are using defmt-tests but the definition macro is only needed once in a project and it is defined in `lib.rs`.
  - Comments explaining directly what reading the line of code does are useless and must be omitted.
  - Comments that describes complex logic inside functions must be added.
Tests:
  - All in-line tests must be HW agnostic so we can run them in QEMU (this is how `cargo test` is configured), so only test code which don't need actuall HW.
  - We can also write HIL (HW in loop) test, but need to use feature flags for these.
  - We also write tests that actually call our code. Tests that does not touch anything outside of test module should be omitted. Also tests like `assert_eq!(20, 20);` are absolute useless.
  - If you are not able to test the actual implementation, ignore those tests and maybe only comment that it is hard to test it.
  - We are using `defmt-test` crate for tests, as this is no-std environment.

We are keeping modules small and separated by application/business logic into submodules:
  - One module should do a one thing
  - We can combine multiple modules and struct for complex functionality (don't forget about modern Rust modules).

We are not re-implementing wheel,
so if there is existing crate we rather use that instead of writing our own implementations.

We are using Embassy framework.

Distributed Testing with defmt-testFor testing individual modules (like parser, sensors, etc.) while using defmt-test, create separate test files in the tests/ directory instead of inline tests in source modules. Each test file becomes its own test binary, avoiding symbol conflicts. Create tests

In main.rs use only defmt logging. In HW module and all its sub-modules use only defmt logging.
In other part of the project which are HS agnostic use usb_logging.
Also use correctly usb log level:
  - Trace for every bit what is the app doing so we can read it as a story.
  - Debug for more verbose logging which is still not suitable for production, but the app shuold not be affected by this.
  - Info logs for what is the app doing which can be enabled in production, so it should not log for every little fucntion. And not too often.
  - Warnings for recoverable conditions and issues.
  - Errors for not handled or not recoverable conditions.


## Project structure

All HW dependent code should be under `hw` module under specific HW.
These should be smallest possible to only implement HW layer.
All these should be also presented in traits in `hw` module.

All HW agnostic code should be outside of `hw` module, and can be dependent on the `hw` traits.
But not directly on specific HW.


## Project description

We are building a firmware for HW implemented in Rust using Embassy framework.
The HW will be implementing communication over 433MHz OOK HW, will be some sensors
(like temperature, humidity and others).
The protocol on top of the 433Mhz will be manchester coded with reed solomon error corrections implementing packets and ACKs.
All HW will be able to send and receive data.
The firmware will implement all HW agnostic implementations outside of HW module.
And specific HW in hw module.
We will implement STM32 blackpill based firmware but also PiPico2 HW which can be easily emulated in QEMU.
The device should be debuggable over serial over USB.


# LLM guidelines

If the project can't be compiled use `cargo check --message-format=json` to verify what are the compilation errors.
Be sure to use the JSON format as it will be much more readable for LLM.


# Documentations and code examples

Use context7 MCP.
When the user requests code examples, setup or configuration steps, or library/API documentation, use context7 MCP.
When investigating currently used crates, consult context7 for correct doccumentation!


# Memory Usage Guidelines

You should use memory tools thoughtfully to enhance conversation continuity and context retention:

## When to Save Memory
- **save_memory**: Store significant conversation exchanges, important decisions, user preferences, or key context that would be valuable to remember in future conversations
- Focus on information that has lasting relevance rather than temporary details
- Save when users share important personal information, project details, or ongoing work context

## When to Update Memory Abstract
- **update_memory_abstract**: After processing recent conversations, combine new important information with existing context to create an improved summary
- Update when there are meaningful developments in ongoing projects or relationships
- Consolidate related information to maintain coherent context over time

## When to Recall Memory
- **recall_memory_abstract**: Use at the beginning of conversations to understand previous context, or when you need background information to better assist the user
- **get_recent_memories**: Access when you need specific details from recent exchanges that aren't captured in the abstract
- Recall when the user references previous conversations or when context would significantly improve your assistance

## What Constitutes Critical Information
- User preferences and working styles
- Ongoing projects and their current status
- Important personal or professional context
- Decisions made and their rationale
- Key relationships or collaborations mentioned
- Technical specifications or requirements for recurring tasks

Use these tools to build continuity and provide more personalized assistance, not as error-prevention mechanisms or intent-guessing systems.