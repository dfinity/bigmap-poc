import React from 'react';
import ReactDOM from 'react-dom';

import Navbar from 'react-bootstrap/Navbar';
import Nav from 'react-bootstrap/Nav';
import Jumbotron from 'react-bootstrap/Jumbotron';
import Table from 'react-bootstrap/Table';
import Container from 'react-bootstrap/Container';
import ChartsPage from './Charts';

// A wide choice of themes available at https://bootswatch.com/
// Here is a shortlist
import "bootswatch/dist/darkly/bootstrap.min.css";
// import "bootswatch/dist/flatly/bootstrap.min.css";
// import 'bootstrap/dist/css/bootstrap.css';

// import tsreact from 'ic:canisters/tsreact_v2';
import { test } from './test';

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
    test();
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
        <Navbar collapseOnSelect expand="lg" bg="dark" variant="dark">
          <Navbar.Brand href="#home">BigMap Dashboard</Navbar.Brand>
          <Navbar.Toggle aria-controls="responsive-navbar-nav" />
          <Navbar.Collapse id="responsive-navbar-nav">
            <Nav className="mr-auto">
              <Nav.Link href="#overview">Overview</Nav.Link>
              <Nav.Link href="#utilization">Utilization</Nav.Link>
              <Nav.Link href="#search">Search</Nav.Link>
            </Nav>
            <Nav>
              <Nav.Link href="https://github.com/dfinity/bigmap-rs">Fork me on GitHub</Nav.Link>
            </Nav>
          </Navbar.Collapse>
        </Navbar>
        <Jumbotron>
          <ChartsPage />
        </Jumbotron>
        <Jumbotron>
          <h1 className="header">BigMap Entries</h1><br />
          <Table striped bordered hover>
            <thead>
              <tr>
                <th>#</th>
                <th>First Name</th>
                <th>Last Name</th>
                <th>Username</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>1</td>
                <td>Mark</td>
                <td>Otto</td>
                <td>@mdo</td>
              </tr>
              <tr>
                <td>2</td>
                <td>Jacob</td>
                <td>Thornton</td>
                <td>@fat</td>
              </tr>
            </tbody>
          </Table>
        </Jumbotron>
      </Container>
    );
  }
}

export default App;

/*
NB: dfx bootstrap's index.html generated looks like this:

<app id="app"><progress class="ic_progress" id="ic-progress">Loading...</progress></app>
*/
ReactDOM.render(<App />, document.getElementById('app'));
