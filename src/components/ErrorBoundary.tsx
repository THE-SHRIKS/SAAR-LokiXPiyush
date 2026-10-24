import React from "react";
import { AlertCircle } from "lucide-react";

interface ErrorBoundaryProps {
  /** Name of the screen, shown in the message. */
  label: string;
  children: React.ReactNode;
}

interface ErrorBoundaryState {
  error: Error | null;
}

/**
 * Keeps a rendering failure in one screen from blanking the whole application. The user sees what
 * failed and can try again; the rest of the shell (navigation, lock) keeps working.
 */
export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error(`Screen "${this.props.label}" failed to render:`, error, info.componentStack);
  }

  render() {
    if (!this.state.error) return this.props.children;
    return (
      <div className="screen-error" role="alert">
        <AlertCircle size={18} />
        <div>
          <strong>The {this.props.label} screen could not be displayed.</strong>
          <p>Your records are unchanged. {this.state.error.message}</p>
          <button type="button" className="btn-secondary" onClick={() => this.setState({ error: null })}>
            Try again
          </button>
        </div>
      </div>
    );
  }
}
