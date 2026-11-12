# @everymail/design

The one source of everything visual that the app and the landing site share. Add the package as `"@everymail/design": "workspace:*"`.

| Piece                                       | Use                                                                                                                                                                           |
| ------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `tokens.json`                               | Source of every colour, space, radius, shadow, duration and easing. Never copy a value; use the CSS variable                                                                  |
| `designPlugin` (`@everymail/design/plugin`) | Vite plugin, in `vite.config.ts` of the app and `vite: { plugins }` of Astro. Serves `virtual:tokens.css` (generated, never committed) and keeps SVG sprites out of data URLs |
| `components.css`                            | The component and motion classes of the design, plus reset and the bundled Geist fonts                                                                                        |
| `logos.svg`, `icons.svg`                    | SVG sprites, referenced with `<use>`                                                                                                                                          |

```ts
import "virtual:tokens.css"; // first: the variables
import "@everymail/design/components.css"; // then the classes, Geist and Geist Mono
```

Theme: the system preference applies by default. `document.documentElement.dataset.theme = "dark" | "light"` overrides it; remove the attribute for System. The theme is engine state; the attribute is only how it is applied. Where View Transitions are missing, add the `theme-fade` class to the root before changing the attribute and remove it when the body's background transition ends: colours crossfade.

Density: `document.documentElement.dataset.density = "compact" | "comfortable" | "roomy"` sets `--row-pad` (`.mrow` padding), `--row-min` (a mail list row's height) and `--menu-row-min` (`.menu-row`). Comfortable is the default.

Sprites: import the file as a URL and point `<use>` at an id. Icons take their colour and stroke from `svg.i` (`sm`, `lg` for sizes) and `currentColor`; logos keep their own colours; `mark` follows the theme.

```tsx
import icons from "@everymail/design/icons.svg?url";
import logos from "@everymail/design/logos.svg?url";

<svg className="i" aria-hidden="true"><use href={`${icons}#search`} /></svg>
<svg width={18} height={18} aria-hidden="true"><use href={`${logos}#gmail`} /></svg>
```

In Astro the same two imports work in the frontmatter.

Icon ids: inbox, calendar, briefing, clock, settings, check, plus, chevron-left, chevron-right, chevron-down, compose, archive, star, send, receipt, quarantine, shield, close, play, retry, search, summary, more, lock, snooze, link-off, arrow-right, forward.

Logo ids: mark, gmail, outlook, icloud, yahoo, fastmail, imap, google, microsoft, gcal, anthropic, openai, gemini, openrouter.

Motion classes follow the rules in `AGENTS.md`. State is a class the engine's view decides: `.in`/`.out` on `.pop`, `.sheet`, `.scrim`, `.drawer-r`; `.hide` on the leaving child of `.stack` (text) or `.swap` (icons); `.stagger` with `style="--i: n"` delays row `n` by 30 ms.

`.stagger` comes after the entrance classes (`.row-in`, `.settle`, `.fade-in`), so it combines with any of them.

Small pieces shared by the app and the site: `.field` (a label, an input, textarea or select, and an optional hint; one tone up on a `.sheet` or `.drawer-r`), `.logo-tile` (a logo on its own tile), `.link` (a text button that reads as a link) and `.sr-only` (read by screen readers, not shown).
