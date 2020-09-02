import React from "react";
import ReactDOM from "react-dom";
import "./index.css";
import App from './App';
import { updateHead } from './Head';

// Importing the Bootstrap CSS
import 'bootstrap/dist/css/bootstrap.min.css';

updateHead(document);

ReactDOM.render(<App />, document.getElementById('root'));
