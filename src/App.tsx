// App root: load the state once, then render the panel.
//
// v2 is intentionally tiny: no routing, no i18n layer, no settings pages -
// just the Light/Dark switch and two small options.

import { useEffect, useState } from "react";
import { api, errorMessage } from "./api";
import { Panel } from "./views/Panel";
import type { AppStateResponse } from "./types";

export default function App() {
  const [state, setState] = useState<AppStateResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let disposed = false;
    (async () => {
      try {
        const s = await api.getState();
        if (!disposed) setState(s);
      } catch (e) {
        if (!disposed) setError(errorMessage(e));
      }
    })();
    return () => {
      disposed = true;
    };
  }, []);

  if (error) {
    return (
      <div className="panel">
        <div className="panel-body">
          <p className="error">Could not read the Windows theme.</p>
          <p className="error-detail">{error}</p>
        </div>
      </div>
    );
  }

  if (!state) {
    return <div className="panel" />;
  }

  return <Panel initial={state} />;
}
