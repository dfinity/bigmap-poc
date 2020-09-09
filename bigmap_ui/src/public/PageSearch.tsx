
import React from 'react';
import Table from 'react-bootstrap/Table';
import { MDBContainer } from "mdbreact";

class PageOverview extends React.Component {
  state = {
  };

  render() {
    return (
      <MDBContainer>
        <h3 className="header">BigMap Entries</h3><br />
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
      </MDBContainer>
    );
  }
}

export default PageOverview;
