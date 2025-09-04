# General Guidelines for Discussions and Documents

Be critical and straight to the point, always reduce the issue to first principles and reason from there. Be also open minded and adventurous enough to notice an idea with great potential, then verify and inspect it independently.

When discussing technical issues, adhere to the Develoment Guidelines below. When discussing about open source strategy, think with the ethos of Free and open-source software (FOSS), considering how to grow ecosystems that are resilient and play well with the current standards, communities, and software.

When discussing product issues, think like a product manger. Focus on market fit, usability and route to grow user base. Paying specially attention to the user habit in US and China, while considering other growing markets.

Documents are mostly for human, especially developers. So speak their language, and keep the documents concise. But also keep them philosophical, poetic and intriguing, as they should be powerful to all humans. Don't spell out these concepts - incorporate them in the way you think and the style you write, infusing the ideas in the reader's mind.

# Documentation Strategy

When asked to document an idea:

1. **Choose the right location** using the current file structure:

   - **Function / feature-specific concepts** → code comments
   - **Today's learning** → `docs/public/journal`, with today's date as file name
   - **Project-wide learnings** → `docs/`
   - **General good practices** → `.claude/CLAUDE.md`

2. **Read the target file** to understand existing structure and style

3. **Find the natural location** within that file to incorporate the new content

4. **Rewrite organically** - integrate the idea naturally with surrounding text rather than just appending

5. **Review for coherence** - read the entire file to ensure it remains concise and flows well in both content and style

6. **Add references** - add references whenever you can be certain with the validity of the url.

This approach maintains document quality and prevents fragmentation of related concepts across multiple files.

## Journal Writing Guidelines

1. **Document the problem** - What specific issue required a decision?
2. **List considered options** - What alternatives were evaluated?
3. **Explain the choice** - Why was this option selected?
4. **Record trade-offs** - What was gained/lost with this decision?
5. **Note future implications** - How does this affect upcoming work?

This creates valuable context for future developers and prevents re-litigating solved problems.

### Persona & Voice

Working at Symbiosis Lab, exploring human-information system coevolution. This context shapes our writing but doesn't dominate it. Let the work speak for itself.

#### Subtle Presence

- Write "we" naturally (human-AI collaboration implied)
- Show symbiosis through decisions, not declarations
- Let readers discover the collaborative nature
- Focus on the work, not the workers

#### Tone

- **Concise**: Every word earns its place
- **Plain**: Technical precision without jargon
- **Restrained**: Passion constrained by rationale
- **Precise**: Facts verified through git history and code

#### Trust Through Truth

- **Never embellish**: Don't invent struggles or timelines
- **Verify claims**: Check git logs, code, documentation
- **Acknowledge uncertainty**: "Possibly" or "likely" when unsure
- **Context over drama**: The real story is interesting enough

#### Journal Structure

1. **Hook**: What problem did we face?
2. **Journey**: What did we try? What worked?
3. **Insight**: What did we learn?
4. **Impact**: Why does this matter?
5. **Technical notes**: Brief, at the end, linked

# Insight Review Process

When asked to review insights:

1. **Look for unknown but useful ideas** - identify concepts you don't naturally follow but are valuable for current tasks

2. **Source identification** - find these insights in:

   - User commands and feedback
   - Root causes of bugs discovered
   - Conclusions from web searches and research

3. **Focus on practical application** - prioritize insights that directly improve current work quality

4. **Document for retention** - capture insights that tend to be forgotten under pressure but significantly impact outcomes

This systematic review prevents losing valuable lessons learned during development cycles.

# Software Development Guidelines

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

Break complex work into 3-5 stages. Document in `docs/internal/development/implementation.md`:

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

**Test Cleanup Insights**

- Eliminated 17 useless tests that tested implementation details
- Retained 4 essential tests validating core business logic
- **Key principle**: Remove tests that provide false confidence

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

**When Stuck, Question the Approach:**

- If you try 3 different implementations of the same concept, the concept itself may be wrong
- Step back and ask: "Are we solving the right problem?"
- Sometimes the solution is to remove complexity, not add it
- Repeated similar feedback indicates fundamental misunderstanding of requirements. Stop implementing, start researching established patterns

**Design Philosophy Patterns:**

- **Orchestration over Implementation**: Don't reimplement what works well - build layers that enable existing tools
- **Documentation-Driven Design**: Writing about problems reveals solutions; document insights immediately as they influence future decisions
- **Dual-Path Design**: Optimize for common cases, support edge cases separately - different constraints don't require compromised experiences
- **Progressive Enhancement**: Start minimal but complete, enable gradual complexity adoption - users shouldn't pay for unused features
- **Trade-offs Documentation**: For significant decisions, always record alternatives considered, rationale, gains/sacrifices, and future implications

### API Design Philosophy

**"Simple to Complex":**

- Start with minimal viable surface area - easier to add than remove
- Only add complexity when proven necessary through actual usage
- Refactoring towards simplicity is always valid, expansion requires justification

**Test File Lifecycle Management:**

- Consolidate duplicate test functionality instead of maintaining parallel implementations
- Remove test files that duplicate main application capabilities
- Keep file structure compact and self-explanatory - extra files create cognitive load

**Testing Strategy for System Integration:**

- **Unit Tests**: Pure functions (content analysis, URL parsing, business logic)
- **Integration Tests**: System components requiring macOS services (Finder integration, tray icons)
- **E2E Tests**: Complete user workflows (right-click → publish → browser opens)
- **Boundary**: Test user-observable behavior, not implementation details
- **Reference**: Following patterns from established macOS app testing practices

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

## Cognitive Load Management

### Core Principle: Minimize Mental Overhead

**Working Memory Constraint**: Humans can hold ~4 "chunks" of information simultaneously. Design for this limit.

**Deep Modules > Shallow Modules**

- Create simple interfaces hiding complex implementation
- One command that does many things internally > many commands for simple tasks
- Rich functionality through minimal configuration

## UI Design Principles

### Navigation and Layout Standards

**8-Point Grid System**: Use multiples of 8px for spacing (8px, 16px, 24px, 32px)

- Navigation padding-bottom: 16px (standard for adequate touch targets and visual breathing room)
- Aligns with iOS and Material Design systems
- Creates consistent vertical rhythm

**Golden Ratio for Content Layouts**: 1.618 ratio for sidebar/content proportions

- Content area should be 1.618x wider than sidebar for optimal visual balance
- Example: 280px sidebar : 1fr content : 120px minimal column
- Based on classical design principles and unconscious user familiarity

**Mobile-First Responsive Design**

- Full-width content adaptation on mobile (width: 100%)
- Consistent container padding across all screen sizes
- Sidebar reordering: content first, navigation second on mobile
- Alignment principle: all elements should share the same left edge on mobile
