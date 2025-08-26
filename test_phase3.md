# Phase 3 Testing Guide

## Multi-Staff Features to Test

### Basic Multi-Staff Operation
1. **Start with 1 staff** (default)
2. **Add staff**: Press `+` to add a new staff (up to 4 total)
3. **Switch staves**: Use ↑/↓ arrows to move between staves
4. **Current staff indication**: Title shows "Staff X/Y"
5. **Visual differentiation**: Current staff uses solid lines (─), others use dots (·)

### Independent Cursors
1. **Per-staff cursors**: Each staff maintains its own cursor position
2. **Cursor movement**: ←/→ arrows move cursor on current staff only
3. **Note placement**: Notes placed only on current staff
4. **Staff switching**: Cursor position preserved when switching staves

### Staff Management
1. **Add staff**: `+` key (maximum 4 staves)
2. **Remove staff**: `-` key (minimum 1 staff)
3. **Auto-adjust**: Current staff adjusts if removed staff was selected

### Persistence
1. **Save multi-staff**: `s` saves all staves and their notes
2. **Load multi-staff**: `l` restores all staves with correct cursor positions

### Test Workflow
1. Start application: `cargo run`
2. Add 2-3 staves with `+`
3. Place different notes on each staff (switch with ↑/↓)
4. Test cursor independence
5. Save with `s`, quit with `q`
6. Restart and load with `l` to verify persistence

## Updated Controls
- ←/→: Move cursor on current staff
- ↑/↓: Switch between staves
- +: Add staff (max 4)
- -: Remove staff (min 1)
- c/d/e/f/g/a/b: Add notes to current staff
- Space: Cycle note duration
- #: Cycle accidentals
- Delete/Backspace: Remove note from current staff
- s: Save score
- l: Load score
- q: Quit