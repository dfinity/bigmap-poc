import React from 'react';
import ReactDOM from 'react-dom';
import { HashRouter, Route } from 'react-router-dom';

import { Container, Jumbotron, Navbar, Nav, Form, FormControl, Button } from "react-bootstrap";
import PageOverview from './PageOverview';
import PageDetails from './PageDetails';
import PageSearch from './PageSearch';
import { updateHead } from './Head';

// A wide choice of themes available at https://bootswatch.com/
// Here is a shortlist
import "bootswatch/dist/darkly/bootstrap.min.css";
// import "bootswatch/dist/flatly/bootstrap.min.css";
// import 'bootstrap/dist/css/bootstrap.css';

// import tsreact from 'ic:canisters/tsreact_v2';
import './App.css';

interface AppState {
  count: number;
}
interface AppProps { }

class App extends React.Component<AppProps, AppState> {
  public state: AppState = {
    count: 0,
  };

  constructor(props: AppProps) {
    super(props);
  }

  public increment = this._updateCount.bind(this, '+');
  public decrement = this._updateCount.bind(this, '-');

  private async _updateCount(val: '+' | '-') {
    let bigIntCount = 0;

    switch (val) {
      case '+':
        // @ts-ignore
        bigIntCount = await tsreact.increment();
        break;
      case '-':
        // @ts-ignore
        bigIntCount = await tsreact.decrement();
        break;
    }
    const count = parseInt(BigInt(bigIntCount).toString(), 10);
    this.setState({ count });
  }

  render() {
    return (
      <Container>
        <h2 className="pt-5 pb-5">BigMap Dashboard</h2>
        <Jumbotron className="pt-1" fluid>
          <Navbar bg="primary" variant="dark" expand="lg">
            {/* <Navbar.Brand href="#home">Home</Navbar.Brand> */}
            <Navbar.Toggle aria-controls="basic-navbar-nav" />
            <Navbar.Collapse id="basic-navbar-nav">
              <Nav className="mr-auto">
                <Nav.Link href="#home">Home</Nav.Link>
                <Nav.Link href="#details">Details</Nav.Link>
                <Nav.Link href="#search">Search</Nav.Link>
              </Nav>
            </Navbar.Collapse>
          </Navbar>
          <Jumbotron className="p-1"></Jumbotron>
          <HashRouter>
            <Route exact path="/" component={PageOverview} />
            <Route path="/home" component={PageOverview} />
            <Route path="/details" component={PageDetails} />
            <Route path="/search" component={PageSearch} />
          </HashRouter>
        </Jumbotron>
      </Container>
    );
  }
}

export default App;

updateHead(document);

/*
NB: dfx bootstrap's index.html generated looks like this:

<app id="app"><progress class="ic_progress" id="ic-progress">Loading...</progress></app>
*/
ReactDOM.render(<App />, document.getElementById('app'));
