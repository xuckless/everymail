// Motion's animation and layout features, loaded after first paint so the first bundle stays lean
// (LazyMotion in shell.tsx). Until they arrive, elements render at their final state.
export { domMax as default } from "motion/react";
