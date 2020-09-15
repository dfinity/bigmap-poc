// Updated required for "mobile web app" behavior
// https://developer.apple.com/library/archive/documentation/AppleApplications/Reference/SafariWebContent/ConfiguringWebApplications/ConfiguringWebApplications.html

// import touchIcon from "../shared/img/bigmap-icon.png";

const APP_TITLE = "BigMap UI";

const updates = [
  // { el: "link", attrs: { rel: "apple-touch-icon", href: touchIcon } },
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

export const updateHead = (document: Document) => {
  document.title = APP_TITLE;

  updates.forEach(({ el, attrs }) => {
    const update: any = document.createElement(el);
    Object.entries(attrs).forEach(([key, value]) => {
      update[key] = value;
    });
    document.getElementsByTagName("head")[0].appendChild(update);
  });
};
