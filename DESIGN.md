---
name: Obsidian Shell
colors:
  surface: '#131319'
  surface-dim: '#131319'
  surface-bright: '#39383f'
  surface-container-lowest: '#0d0e13'
  surface-container-low: '#1b1b21'
  surface-container: '#1f1f25'
  surface-container-high: '#2a2930'
  surface-container-highest: '#34343b'
  on-surface: '#e4e1ea'
  on-surface-variant: '#c1c6d4'
  inverse-surface: '#e4e1ea'
  inverse-on-surface: '#303036'
  outline: '#8b919e'
  outline-variant: '#414752'
  surface-tint: '#a7c8ff'
  primary: '#a7c8ff'
  on-primary: '#003061'
  primary-container: '#4691f2'
  on-primary-container: '#002a55'
  inverse-primary: '#005eb2'
  secondary: '#53df98'
  on-secondary: '#003920'
  secondary-container: '#00af6d'
  on-secondary-container: '#003920'
  tertiary: '#f3c00d'
  on-tertiary: '#3d2e00'
  tertiary-container: '#d2a600'
  on-tertiary-container: '#503d00'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#d5e3ff'
  primary-fixed-dim: '#a7c8ff'
  on-primary-fixed: '#001b3c'
  on-primary-fixed-variant: '#004788'
  secondary-fixed: '#73fcb2'
  secondary-fixed-dim: '#53df98'
  on-secondary-fixed: '#002111'
  on-secondary-fixed-variant: '#005231'
  tertiary-fixed: '#ffdf90'
  tertiary-fixed-dim: '#f3c00c'
  on-tertiary-fixed: '#241a00'
  on-tertiary-fixed-variant: '#584400'
  background: '#131319'
  on-background: '#e4e1ea'
  surface-variant: '#34343b'
typography:
  display:
    fontFamily: Inter
    fontSize: 2rem
    fontWeight: '700'
    lineHeight: 2.5rem
    letterSpacing: -0.02em
  headline-lg:
    fontFamily: Inter
    fontSize: 1.5rem
    fontWeight: '600'
    lineHeight: 2rem
    letterSpacing: -0.015em
  headline-md:
    fontFamily: Inter
    fontSize: 1.25rem
    fontWeight: '600'
    lineHeight: 1.75rem
    letterSpacing: -0.01em
  title-sm:
    fontFamily: Inter
    fontSize: 1rem
    fontWeight: '600'
    lineHeight: 1.5rem
    letterSpacing: -0.005em
  body-lg:
    fontFamily: Inter
    fontSize: 1rem
    fontWeight: '400'
    lineHeight: 1.5rem
  body-md:
    fontFamily: Inter
    fontSize: 0.875rem
    fontWeight: '400'
    lineHeight: 1.25rem
  body-sm:
    fontFamily: Inter
    fontSize: 0.75rem
    fontWeight: '400'
    lineHeight: 1rem
  label-lg:
    fontFamily: Inter
    fontSize: 0.875rem
    fontWeight: '500'
    lineHeight: 1.25rem
  label-md:
    fontFamily: Inter
    fontSize: 0.75rem
    fontWeight: '600'
    lineHeight: 1rem
    letterSpacing: 0.02em
  label-mono:
    fontFamily: JetBrains Mono
    fontSize: 0.75rem
    fontWeight: '500'
    lineHeight: 1rem
    letterSpacing: '0'
rounded:
  sm: 0.25rem
  DEFAULT: 0.5rem
  md: 0.75rem
  lg: 1rem
  xl: 1.5rem
  full: 9999px
spacing:
  gutter: 0.75rem
  gutter-desktop: 1rem
  margin: 1rem
  margin-desktop: 1.5rem
  space-xs: 0.25rem
  space-sm: 0.5rem
  space-md: 0.75rem
  space-lg: 1rem
  space-xl: 1.5rem
---

## Brand & Style

This design system targets power users, workstation professionals, and Linux desktop enthusiasts who value precision, system transparency, and distraction-free visual ergonomics. The identity fuses the understated utility of upstream GNOME Adwaita with the tactile elegance of elementaryOS Granite and KDE Breeze's micro-surfaces.

The design movement combines **Tonal Precision Minimalism** with refined **Slight Glass / Atmospheric Layering**. Interfaces prioritize immediate legibility of hardware states, audio telemetry, network performance, and system toggles without gratuitous ornamentation. Surfaces do not rely on aggressive blurs; instead, they employ deeply graded dark obsidian plates, micro-borders with sub-pixel luminance, and high-visibility status accents. The emotional response is centered on operational control, high software craftsmanship, and quiet confidence.

## Colors

The palette is engineered specifically for dark desktop environments, maintaining low visual fatigue while preserving instant recognition of system events and interactive states.

### Core Roles
- **Primary Accent (`#3584e4`)**: Canonical system blue. Used for active quick switches, slider fills, primary action triggers, focused states, and selected radio/checkbox indicators.
- **Secondary / Success (`#2ec27e`)**: System emerald. Reserved for battery health, secure connection indicators (VPN, SSH), completed operations, and active hardware switches.
- **Tertiary / Warning (`#f5c211`)**: Vibrant amber. Used for thermal warnings, performance throttling thresholds, power-saver states, and captive portal notifications.
- **Destructive (`#e01b24`)**: High-contrast red. Used for power-off actions, session resets, unmounted drives, and device disconnections.
- **Focus Ring / Discovery (`#9141ac`)**: Deep system purple. Applied exclusively to persistent keyboard navigation focus rings and specialized hardware pairing modes.

### Surface Architecture
- **Base Canvas (`#18181e`)**: Root window canvas, system wallpaper backdrop margins, and deep application shell bases.
- **Surface Elevation 1 (`#1e1e24`)**: Default client-side decoration (CSD) headerbars, panel drawers, and grouped container trays.
- **Surface Elevation 2 (`#282832`)**: Modular widget cards, floating quick-toggle tiles, list row containers, and popover bases.
- **Surface Elevation 3 (`#32323e`)**: Hover states, active slider thumbs, pressed card states, and embedded input wells.
- **Border Trim (`rgba(255, 255, 255, 0.08)`)**: Universal hair-line separator delivering crisp geometry without visual weight.
- **Active Border Trim (`rgba(255, 255, 255, 0.16)`)**: Applied to cards and interactive controls upon hover or selection.

## Typography

The type system is built on **Inter** for high-density legibility across varying display DPIs, supplemented by **JetBrains Mono** for hardware statistics, IP addresses, memory consumption, and port metrics.

### Typography Roles
- **Display & Headline Levels**: Employed strictly in the primary System Settings window overview, full-page empty states, and section landing zones.
- **Title & Label Levels**: Form the backbone of Quick Settings tiles, setting row identifiers, and headerbar window titles. All label levels maintain crisp hinting and balanced mid-weights (`500` and `600`) to avoid dark-theme halation.
- **Label Mono**: Used across hardware readout modules (e.g., `4.2 GHz`, `12.4 GB / 32 GB`, `eth0: 192.168.1.42`) and slider percentage tooltips.

## Layout & Spacing

Layout geometry follows an explicit, mathematical 8px base grid with a 4px sub-rhythm for micro-controls (e.g., switches, badges, sliders).

### Layout Modes
1. **Full-Window Client-Side Decoration (CSD / Headerbar)**:
   - Top Headerbar height: fixed at `48px`. Integrates window controls (close, minimize, maximize) on the title line, search entry, and segmented views.
   - Dual-pane layout: Fixed-width left navigation sidebar (`260px`) linked with a fluid content canvas (`min-width: 520px`).
   - Inner content conforming to an 8-column layout with `gutter-desktop: 1rem` and `margin-desktop: 1.5rem`.

2. **Floating Quick Settings Tray / Notification Shell**:
   - Anchored popover fixed at `width: 380px` or `420px` depending on screen viewport scale.
   - Structured in an internal 2-column or 4-column compact modular grid with `gutter: 0.75rem`.
   - Vertical flow separated by category clusters (Network/Display/Audio/Power) using `space-md` gaps.

## Elevation & Depth

Visual hierarchy is constructed through **Tonal Stacking** reinforced by **Sub-pixel Rim Lighting**, rather than heavy blur or diffuse black drop-shadows.

### Elevation Hierarchy
- **Level 0 (Canvas Base - `#18181e`)**: No shadow. Completely neutral background.
- **Level 1 (Window & Shell - `#1e1e24`)**: 
  - Rim light: `inset 0 1px 0 0 rgba(255, 255, 255, 0.08)`.
  - Outer boundary: `1px solid rgba(255, 255, 255, 0.08)`.
  - Ambient shadow: `0 8px 24px rgba(0, 0, 0, 0.45)`.
- **Level 2 (Interactive Tiles, Sliders & Inset Lists - `#282832`)**:
  - Rim light: `inset 0 1px 0 0 rgba(255, 255, 255, 0.05)`.
  - Border: `1px solid rgba(255, 255, 255, 0.06)`.
  - Ambient shadow: `0 2px 6px rgba(0, 0, 0, 0.25)`.
- **Level 3 (Floating Popovers, Modal Dialogs & Dropdowns - `#282832`)**:
  - Backdrop filter: `blur(20px) saturate(140%)`.
  - Background fill: `rgba(40, 40, 50, 0.92)`.
  - Rim light: `inset 0 1px 0 0 rgba(255, 255, 255, 0.12)`.
  - Outer shadow: `0 16px 36px rgba(0, 0, 0, 0.55), 0 0 0 1px rgba(255, 255, 255, 0.08)`.

## Shapes

The design system uses a deliberate hierarchy of corner curvature to signal structural boundaries versus action points:

- **Window Shell & Main Popovers**: `rounded-2xl` (`1.25rem` / `20px`) for soft modern desktop window frames that seamlessly blend into Wayland/X11 compositors.
- **Modular Cards & Quick Settings Tiles**: `rounded-lg` (`1rem` / `16px`) delivering distinct separation between grouped setting blocks.
- **List Groups & Embedded Rows**: Outer grouped container uses `rounded-lg` (`1rem`), with interior list rows maintaining squared connections and dividing keylines.
- **Pill Toggles, Buttons, Badges, and Slider Thumbs**: Fully circular or pill-shaped (`9999px`) for physical, tactile ergonomics.

## Components

### Headerbar (CSD)
- Height: `48px`. Background: `#1e1e24`.
- Border-bottom: `1px solid rgba(255, 255, 255, 0.08)`.
- Window Controls: Embedded circular buttons (`16px` diameter) or Adwaita-style subtle vector icons (`close`, `minimize`, `maximize`) with low resting opacity (`0.7`) and active states scaling up contrast.

### Quick Settings Modular Tiles (Adwaita 45+ Style)
- Dimensions: Height `56px`, width `100%`.
- Shape: `rounded-lg` (`16px`).
- State: Inactive tiles sit at `#282832` with text at `rgba(255, 255, 255, 0.7)`. Active tiles switch background to `#3584e4`, text to `#ffffff`, and icon fill to `#ffffff`.
- Split Tiles (Action + Menu Chevron): Divided by a subtle internal vertical separator `rgba(255, 255, 255, 0.1)`. Primary tap area triggers state; chevron right opens deep popover settings.

### Continuous Sliders (Audio & Display Brightness)
- Height: `40px` thick capsule track.
- Background: Base trough at `#1e1e24` with `1px solid rgba(255, 255, 255, 0.06)`.
- Fill Track: Active color (`#3584e4` or neutral white) expanding from left to right with rounded terminus.
- Left-anchored iconography (e.g., volume-low, brightness-low) overlaid directly inside the track with high contrast.
- Telemetry readout: `label-mono` right-aligned badge floating outside or inside the track end.

### System List Rows
- Container: Grouped card with `#282832` fill and `rgba(255, 255, 255, 0.08)` outer border.
- Row structure: Height `52px`, horizontal padding `space-lg`.
- Dividers: `1px solid rgba(255, 255, 255, 0.04)` inset by `48px` to align with text after leading icons.
- Right Accessories: Secondary label (`body-sm`), chevron, or pill switch.

### Pill Switches
- Dimensions: `44px` width, `26px` height.
- Track: `#18181e` inactive; `#3584e4` active.
- Thumb: `20px` diameter circle, bright `#ffffff`, resting with a `3px` offset. Smooth transition: `cubic-bezier(0.16, 1, 0.3, 1) 200ms`.

### Badges & Hardware Indicators
- Padding: `0.125rem 0.5rem`.
- Font: `label-mono` / `0.75rem`.
- Variants:
  - Green (`#2ec27e` on `rgba(46, 194, 126, 0.15)`): Online, Battery Healthy, VPN Active.
  - Amber (`#f5c211` on `rgba(245, 194, 17, 0.15)`): Balanced Power, Updates Pending.
  - Red (`#e01b24` on `rgba(224, 27, 36, 0.15)`): Performance Mode, Low Battery (<15%).