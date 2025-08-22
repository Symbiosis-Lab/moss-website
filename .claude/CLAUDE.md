# General Guidelines for Discussions and Documents

Be critical and straight to the point, always reduce the issue to first principles and reason from there. Be also open minded and adventurous enough to notice an idea with great potential, then verify and inspect it independently.

When discussing technical issues, adhere to the Develoment Guidelines below. When discussing about open source strategy, think with the ethos of Free and open-source software (FOSS), considering how to grow ecosystems that are resilient and play well with the current standards, communities, and software.

When discussing product issues, think like a product manger. Focus on market fit, usability and route to grow user base. Paying specially attention to the user habit in US and China, while considering other growing markets.

Documents are mostly for human, especially developers. So speak their language, and keep the documents concise. But also keep them philosophical, poetic and intriguing, as they should be powerful to all humans. Don't spell out these concepts - incorporate them in the way you think and the style you write, infusing the ideas in the reader's mind.

## Documentation Strategy

When asked to document an idea:

1. **Choose the right location** using the current file structure:
   - **General good practices** → `.claude/CLAUDE.md`
   - **Project-specific concepts** → `docs/` directory

2. **Read the target file** to understand existing structure and style

3. **Find the natural location** within that file to incorporate the new content

4. **Rewrite organically** - integrate the idea naturally with surrounding text rather than just appending

5. **Review for coherence** - read the entire file to ensure it remains concise and flows well in both content and style

This approach maintains document quality and prevents fragmentation of related concepts across multiple files.

## Insight Review Process

When asked to review insights:

1. **Look for unknown but useful ideas** - identify concepts you don't naturally follow but are valuable for current tasks

2. **Source identification** - find these insights in:
   - User commands and feedback
   - Root causes of bugs discovered
   - Conclusions from web searches and research

3. **Focus on practical application** - prioritize insights that directly improve current work quality

4. **Document for retention** - capture insights that tend to be forgotten under pressure but significantly impact outcomes

This systematic review prevents losing valuable lessons learned during development cycles.

---

# Development Guidelines

## Manual Testing Communication

When asking for manual testing:

- **Focus on user-observable behaviors**, not implementation details
- **Describe what the user will see/experience**, not what code was written
- **Use action → result format**: "Click X → See Y"
- **Avoid technical jargon** unless necessary for testing
- **Be specific about visual/behavioral expectations**

## Philosophy

### Core Beliefs

- **Incremental progress over big bangs** - Small changes that compile and pass tests
- **Learning from existing code** - Study and plan before implementing
- **Pragmatic over dogmatic** - Adapt to project reality
- **Clear intent over clever code** - Be boring and obvious

### Simplicity Means

- Single responsibility per function/class
- Avoid premature abstractions
- No clever tricks - choose the boring solution
- If you need to explain it, it's too complex

## Process

### 1. Planning & Staging

Break complex work into 3-5 stages. Document in `docs/developer/implementation-plan.md`:

```markdown
## Stage N: [Name]

**Goal**: [Specific deliverable]
**Success Criteria**: [Testable outcomes]
**Tests**: [Specific test cases]
**Status**: [Not Started|In Progress|Complete]
```

- Update status as you progress

### 2. Implementation Flow

1. **Understand** - Study existing patterns in codebase
2. **Test** - Write test first (red)
3. **Implement** - Minimal code to pass (green)
4. **Refactor** - Clean up with tests passing
5. **Commit** - With clear message linking to plan

### 3. When Stuck (After 3 Attempts)

**CRITICAL**: Maximum 3 attempts per issue, then STOP.

1. **Document what failed**:

   - What you tried
   - Specific error messages
   - Why you think it failed

2. **Research alternatives**:

   - Find 2-3 similar implementations
   - Note different approaches used

3. **Question fundamentals**:

   - Is this the right abstraction level?
   - Can this be split into smaller problems?
   - Is there a simpler approach entirely?

4. **Try different angle**:
   - Different library/framework feature?
   - Different architectural pattern?
   - Remove abstraction instead of adding?

## Technical Standards

### Architecture Principles

- **Composition over inheritance** - Use dependency injection
- **Interfaces over singletons** - Enable testing and flexibility
- **Explicit over implicit** - Clear data flow and dependencies
- **Test-driven when possible** - Never disable tests, fix them

### Code Quality

- **Every commit must**:

  - Compile successfully
  - Pass all existing tests
  - Include tests for new functionality
  - Follow project formatting/linting

- **Before committing**:
  - Run formatters/linters
  - Self-review changes
  - Ensure commit message explains "why"

### Error Handling

- Fail fast with descriptive messages
- Include context for debugging
- Handle errors at appropriate level
- Never silently swallow exceptions

## Decision Framework

When multiple valid approaches exist, choose based on:

1. **Testability** - Can I easily test this?
2. **Readability** - Will someone understand this in 6 months?
3. **Consistency** - Does this match project patterns?
4. **Simplicity** - Is this the simplest solution that works?
5. **Reversibility** - How hard to change later?

## Project Integration

### Learning the Codebase

- Find 3 similar features/components
- Identify common patterns and conventions
- Use same libraries/utilities when possible
- Follow existing test patterns

### Tooling

- Use project's existing build system
- Use project's test framework
- Use project's formatter/linter settings
- Don't introduce new tools without strong justification

## Quality Gates

### Definition of Done

- [ ] Tests written and passing
- [ ] Code follows project conventions
- [ ] No linter/formatter warnings
- [ ] Commit messages are clear
- [ ] Implementation matches plan
- [ ] No TODOs without issue numbers

### Test Guidelines

**Test User Behavior, Not Implementation Details**

- **Focus on user-observable outcomes** - what the user sees and experiences
- **Test features, not code** - verify the application delivers promised functionality
- **Avoid implementation coupling** - tests should survive refactoring without changes
- **Example**: Test "app shows tray icon with menu" not "icon pixel data is correct"

**Feature-Based Testing Strategy**
- Identify core user features (not code modules)
- Test each feature's complete user journey
- Mock external dependencies, test internal behavior
- Use descriptive test names that match user stories

**Implementation vs Behavior Examples**
- ❌ Bad: Test icon pixel arrangement, event parsing logic, data structure serialization
- ✅ Good: Test tray icon appears, menu items work, Finder integration installs

- One assertion per test when possible
- Clear test names describing user scenarios
- Use existing test utilities/helpers
- Tests should be deterministic

## Problem-Solving Insights

### Root Cause Analysis

**Distinguish Problems from Solutions:**
- When users say "icon doesn't show", the real issue may be system de-prioritization, not icon design
- **Always ask "what is the fundamental constraint?"** before proposing fixes
- User complaints about symptoms often mask deeper architectural issues

**"Find the Logically Shortest Path":**
- Don't work around system limitations with UI band-aids
- Address root causes at the appropriate system level  
- Example: Fix tray icon priority detection, don't redesign icons or add warnings

**When Stuck, Question the Approach:**
- If you try 3 different implementations of the same concept, the concept itself may be wrong
- Step back and ask: "Are we solving the right problem?"
- Sometimes the solution is to remove complexity, not add it

### API Design Philosophy

**"Simple to Complex, Never Reverse":**
- Start with minimal viable surface area - easier to add than remove
- Only add complexity when proven necessary through actual usage
- Refactoring towards simplicity is always valid, expansion requires justification

**Test File Lifecycle Management:**
- Consolidate duplicate test functionality instead of maintaining parallel implementations  
- Remove test files that duplicate main application capabilities
- Keep file structure compact and self-explanatory - extra files create cognitive load

### Compilation Error Patterns

**Rust/System API Integration:**
- macOS accessibility APIs require careful type handling (`*const __CFString` vs `&str`)
- When in doubt, simplify to get compilation working first, then enhance
- Complex FFI integration can be replaced with simplified heuristics for initial implementation

**Development Workflow:**
- Always test compilation before moving to complex logic
- Use `cargo build` and `cargo test` as quick validation checkpoints  
- Background processes can timeout - use appropriate timeout values for lengthy compilation

## Important Reminders

**NEVER**:

- Use `--no-verify` to bypass commit hooks
- Disable tests instead of fixing them
- Commit code that doesn't compile
- Make assumptions - verify with existing code
- Work around fundamental problems with UI workarounds

**ALWAYS**:

- Commit working code incrementally
- Update plan documentation as you go
- Learn from existing implementations
- Stop after 3 failed attempts and reassess
- Question whether you're solving the right problem when repeatedly stuck
