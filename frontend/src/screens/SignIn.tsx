import "./sign-in.css";
import { match, useSend } from "../engine";
import { SignInProvider } from "../gen/ui/v1/common_pb";
import type { SignInView } from "../gen/ui/v1/sign_in_pb";
import { Button } from "../ui/Button";
import { Logo } from "../ui/Logo";
import { Overlap } from "../ui/Overlap";
import { stagger } from "../ui/stagger";

const providers = [
	{ provider: SignInProvider.GOOGLE, label: "Continue with Google" },
	{ provider: SignInProvider.MICROSOFT, label: "Continue with Microsoft" },
];

// The two ways in. While the engine signs in with one, its button says what it waits for, both are
// disabled and Cancel stops waiting; a sign-in that did not finish says why under them.
export function SignIn({ view }: { view: SignInView }) {
	const send = useSend();
	const { state } = view;
	return (
		<div className="sign-in">
			<div className="sign-in-hero settle" style={stagger(0)}>
				<Logo brand={{ kind: "mark" }} size={52} />
				<h1 className="h1 sign-in-title">EveryMail</h1>
				<p className="muted sign-in-lede">
					Every inbox in one place, with agents that work while you don’t
				</p>
			</div>
			<div className="sign-in-actions settle" style={stagger(2)}>
				{providers.map(({ provider, label }) => (
					<Button
						key={provider}
						variant="soft"
						size="lg"
						disabled={signingIn(state)}
						onClick={() =>
							send({ case: "continueWithProvider", value: { provider } })
						}
					>
						<Logo brand={{ kind: "signIn", provider }} size={20} />
						<Overlap
							id={signingWith(state, provider) ? "waiting" : "idle"}
							className="btn-label"
						>
							{match(state, "SignInView.state", {
								idle: () => label,
								failed: () => label,
								signingIn: (signing) =>
									signing.provider === provider ? signing.text : label,
							})}
						</Overlap>
					</Button>
				))}
				{match(state, "SignInView.state", {
					idle: () => null,
					failed: (failed) => (
						<span className="sm bad settle" role="alert">
							{failed.reason}
						</span>
					),
					signingIn: () => (
						<Button
							variant="ghost"
							className="settle"
							onClick={() => send({ case: "cancelSignIn", value: {} })}
						>
							Cancel
						</Button>
					),
				})}
			</div>
		</div>
	);
}

const signingIn = (state: SignInView["state"]) =>
	match(state, "SignInView.state", {
		idle: () => false,
		failed: () => false,
		signingIn: () => true,
	});

const signingWith = (state: SignInView["state"], provider: SignInProvider) =>
	match(state, "SignInView.state", {
		idle: () => false,
		failed: () => false,
		signingIn: (signing) => signing.provider === provider,
	});
