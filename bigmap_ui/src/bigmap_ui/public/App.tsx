import React, { useState } from 'react';
import ReactDOM from 'react-dom';

export default ChartsPage;

import Navbar from 'react-bootstrap/Navbar';
import Nav from 'react-bootstrap/Nav';
import NavDropdown from 'react-bootstrap/NavDropdown';
import Jumbotron from 'react-bootstrap/Jumbotron';
import Table from 'react-bootstrap/Table';
import Container from 'react-bootstrap/Container';
import Button from 'react-bootstrap/Button';

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
    const { count } = this.state;

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

import { Line } from "react-chartjs-2";
import { MDBContainer } from "mdbreact";

class ChartsPage extends React.Component {
  state = {
    dataLine: {
      labels: ["January", "February", "March", "April", "May", "June", "July"],
      datasets: [
        {
          label: "Used Gigabytes",
          fill: true,
          lineTension: 0.3,
          backgroundColor: "rgba(225, 204,230, .3)",
          borderColor: "rgb(205, 130, 158)",
          borderCapStyle: "butt",
          borderDash: [],
          borderDashOffset: 0.0,
          borderJoinStyle: "miter",
          pointBorderColor: "rgb(205, 130,1 58)",
          pointBackgroundColor: "rgb(255, 255, 255)",
          pointBorderWidth: 10,
          pointHoverRadius: 5,
          pointHoverBackgroundColor: "rgb(0, 0, 0)",
          pointHoverBorderColor: "rgba(220, 220, 220,1)",
          pointHoverBorderWidth: 2,
          pointRadius: 1,
          pointHitRadius: 10,
          data: [10, 20, 22, 23, 56, 70, 110]
        }
      ]
    }
  };

  render() {
    return (
      <MDBContainer>
        <h3 className="mt-5">Total used Gigabytes over time</h3>
        <Line data={this.state.dataLine} options={{ responsive: true }} />
      </MDBContainer>
    );
  }
}

/*
NB: dfx bootstrap's index.html generated looks like this:

<app id="app"><progress class="ic_progress" id="ic-progress">Loading...</progress></app>
*/
ReactDOM.render(<App />, document.getElementById('app'));
