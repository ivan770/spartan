(function() {var implementors = {};
implementors["spartan"] = [{"text":"impl&lt;E&gt; From&lt;E&gt; for ResponseError <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;E: RespondableError,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl From&lt;BuilderError&gt; for QueueError","synthetic":false,"types":[]},{"text":"impl From&lt;Error&gt; for PersistenceError","synthetic":false,"types":[]},{"text":"impl From&lt;Box&lt;ErrorKind, Global&gt;&gt; for PrimaryError","synthetic":false,"types":[]},{"text":"impl From&lt;Error&gt; for PrimaryError","synthetic":false,"types":[]},{"text":"impl From&lt;Error&gt; for ReplicaError","synthetic":false,"types":[]},{"text":"impl From&lt;Box&lt;ErrorKind, Global&gt;&gt; for ReplicaError","synthetic":false,"types":[]},{"text":"impl From&lt;PersistenceError&gt; for ReplicaError","synthetic":false,"types":[]},{"text":"impl From&lt;Message&gt; for DeleteResponse","synthetic":false,"types":[]},{"text":"impl&lt;'m&gt; From&lt;&amp;'m Message&gt; for PopResponse&lt;'m&gt;","synthetic":false,"types":[]},{"text":"impl From&lt;usize&gt; for SizeResponse","synthetic":false,"types":[]},{"text":"impl From&lt;Error&gt; for ServerError","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()