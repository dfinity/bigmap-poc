import React from 'react';
import ReactDOM from 'react-dom';
import './index.css';
import App from './public/App';

import "bootswatch/dist/darkly/bootstrap.min.css";

// Updated required for "mobile web app" behavior
// https://developer.apple.com/library/archive/documentation/AppleApplications/Reference/SafariWebContent/ConfiguringWebApplications/ConfiguringWebApplications.html

const APP_TITLE = "BigMap UI";

const updates = [
  {
    el: "meta",
    attrs: { name: "apple-mobile-web-app-title", content: APP_TITLE },
  },
  {
    el: "meta",
    attrs: { name: "apple-mobile-web-app-capable", content: "yes" },
  },
  {
    el: "meta",
    attrs: {
      name: "apple-mobile-web-app-status-bar-style",
      content: "black",
    },
  },
];

const updateHead = (document) => {
  document.title = APP_TITLE;

  updates.forEach(({ el, attrs }) => {
    const update = document.createElement(el);
    Object.entries(attrs).forEach(([key, value]) => {
      update[key] = value;
    });
    document.getElementsByTagName("head")[0].appendChild(update);
  });
};

updateHead(document);

ReactDOM.render(
  <App />,
  document.getElementById('app')
);
