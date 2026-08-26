export function OAuthButtons() {
  return (
    <div className="flex flex-col gap-2">
      <p className="oauth-divider">or continue with</p>
      <div className="flex gap-2">
        <a
          href="/auth/twitch"
          className="oauth-btn text-white"
          style={{ backgroundColor: "#9146FF" }}
        >
          Twitch
        </a>
        <a
          href="/auth/discord"
          className="oauth-btn text-white"
          style={{ backgroundColor: "#5865F2" }}
        >
          Discord
        </a>
      </div>
    </div>
  );
}
