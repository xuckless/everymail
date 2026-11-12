// The brands the logo sprite draws, by what they are to EveryMail, and the one place their names
// are listed for copy (the app's "add a key" and calendar notes, the landing page).

// Bring-your-own-key model providers (AGENTS.md, Stack: LLM access).
export const modelProviders = ["Anthropic", "OpenAI", "Gemini", "OpenRouter"];

// Calendars that join with an account: Google and Outlook directly, the rest over CalDAV.
export const calendarProviders = {
	direct: ["Google", "Outlook"],
	caldav: ["iCloud", "Fastmail"],
};

// "A, B, C or D"
export const anyOf = (names: string[]) =>
	names.length < 2
		? names.join("")
		: `${names.slice(0, -1).join(", ")} or ${names.at(-1)}`;

// "A, B and C"
export const allOf = (names: string[]) =>
	names.length < 2
		? names.join("")
		: `${names.slice(0, -1).join(", ")} and ${names.at(-1)}`;
