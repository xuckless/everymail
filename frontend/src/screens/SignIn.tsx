import "./sign-in.css";
import { useSend } from "../engine";
import { SignInProvider } from "../gen/ui/v1/common_pb";
import type { SignInView } from "../gen/ui/v1/sign_in_pb";
import { Button } from "../ui/Button";
import { Logo } from "../ui/Logo";
import { stagger } from "../ui/stagger";

const providers = [
	{ provider: SignInProvider.GOOGLE, label: "Continue with Google" },
	{ provider: SignInProvider.MICROSOFT, label: "Continue with Microsoft" },
];

// The view carries no data yet: the session is a later roadmap task.
export function SignIn(_: { view: SignInView }) {
	const send = useSend();
	return (
		<div className="sign-in">
			<div className="sign-in-hero settle stagger" style={stagger(0)}>
				<Logo brand={{ kind: "mark" }} size={52} />
				<h1 className="h1 sign-in-title">EveryMail</h1>
				<p className="muted sign-in-lede">
					Every inbox in one place, with agents that work while you don’t
				</p>
			</div>
			<div className="sign-in-actions settle stagger" style={stagger(2)}>
				{providers.map(({ provider, label }) => (
					<Button
						key={provider}
						variant="soft"
						size="lg"
						onClick={() =>
							send({ case: "continueWithProvider", value: { provider } })
						}
					>
						<Logo brand={{ kind: "signIn", provider }} size={20} />
						{label}
					</Button>
				))}
			</div>
		</div>
	);
}
