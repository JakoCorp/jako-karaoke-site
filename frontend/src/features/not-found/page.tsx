import { Link } from "react-router";

export function NotFoundPage() {
  return (
    <div className="not-found">
      <span className="not-found-code">404</span>
      <h1 className="not-found-title">Page not found</h1>
      <p className="not-found-desc">This page doesn't exist or may have been moved.</p>
      <Link to="/" className="btn btn-primary">
        Go home
      </Link>
    </div>
  );
}
