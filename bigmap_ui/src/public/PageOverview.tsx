
import React from 'react';
import Card from 'react-bootstrap/Card';
import { Container, Row, Col, Badge } from "react-bootstrap";

class PageOverview extends React.Component {
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
      <Container>
        <Row>
          <Col>
            <Card className="m-1">
              <Card.Body>
                <Card.Title>Status</Card.Title>
                <Card.Text>
                  <h1><Badge variant="success">Good</Badge></h1>
                </Card.Text>
              </Card.Body>
            </Card>
          </Col>
          <Col>
            <Card className="m-1">
              <Card.Body>
                <Card.Title>Data Bucket Canisters</Card.Title>
                <Card.Text>
                  <h1><Badge variant="secondary">1</Badge></h1>
                </Card.Text>
              </Card.Body>
            </Card>
          </Col>
          <Col>
            <Card className="m-1">
              <Card.Body>
                <Card.Title>Total Data Stored</Card.Title>
                <Card.Text>
                  <h1><Badge variant="primary">20 MB</Badge></h1>
                </Card.Text>
              </Card.Body>
            </Card>
          </Col>
        </Row>
      </Container>
    );
  }
}

export default PageOverview;