# Chat Input Layout Design (Pinned Bottom)

## Goal
Fix the chat input so it is docked to the bottom of the chat window and provides at least 4 visible lines of typing space, without obscuring recent messages.

## Context
- Current chat input appears too high and too short for comfortable typing.
- User wants the composer pinned to the bottom (ChatGPT-style).

## Design Summary
- Dock the composer to the bottom edge of the chat panel.
- Ensure textarea shows 4 lines minimum, with readable padding and line height.
- Reserve bottom space in the message list so the last message is always visible above the composer.

## Layout Behavior
- **Composer position:** Fixed at bottom of `.chat-main`.
- **Message area:** Fills remaining vertical space and scrolls independently.
- **Visibility:** Composer never overlaps the last message; message list has bottom padding equal to composer height.

## Composer Dimensions
- **Minimum height:** 4 visible lines.
- **Typography:** line-height ~1.6 with comfortable padding.
- **Auto-resize:** Can grow up to a max height but never collapse below 4 lines.

## Implementation Notes
- CSS adjusts `.chat-input`, `.chat-input-area`, and `.chat-messages`.
- JS ensures `rows=4` and auto-resize respects minimum height.
- Message list gets bottom padding to match composer height.

## Testing
- Input is always at the bottom edge on all window sizes.
- Four lines are visible without scrolling inside the textarea.
- Latest chat message remains visible and not hidden behind the composer.
