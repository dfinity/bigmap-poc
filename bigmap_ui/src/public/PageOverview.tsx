
import React from 'react';
import Card from 'react-bootstrap/Card';
import { Container, Row, Col, Badge } from "react-bootstrap";
import { getBigMapStatus } from '../utils';
const prettyBytes = require('pretty-bytes');


interface DataBucketUsage {
  canister_id?: string;
  used_bytes?: number;
}

interface BigMapStatus {
  data_buckets?: DataBucketUsage[];
  used_bytes_total?: number;
}

class PageOverview extends React.Component {
  public state: BigMapStatus = {
    data_buckets: [],
    used_bytes_total: 0,
  };

  private timerID: any;

  constructor(props: any) {
    super(props);
    this.state = {
      data_buckets: [],
      used_bytes_total: 0
    };
  }

  componentDidMount() {
    this.refreshData();
    this.timerID = setInterval(() => this.refreshData(), 5000);
  }

  componentWillUnmount() {
    clearInterval(this.timerID);
  }

  private refreshData() {
    getBigMapStatus()
      .then(res => JSON.parse(res))
      .then(json => {
        console.log("got status json:", json);
        this.setState({
          data_buckets: json["data_buckets"],
          used_bytes_total: json["used_bytes_total"]
        });
        console.log(this.state);
      });
  }

  private statusAsText() {
    if (this.state.data_buckets && this.state.data_buckets.length > 0)
      return "Good";
    else {
      return "Unknown"
    }
  }

  private statusAsVariant() {
    if (this.statusAsText() === "Unknown") {
      return "secondary"
    } else {
      return "success"
    }
  }

  private numDataBuckets() {
    // ts-ignore
    return this.state.data_buckets && this.state.data_buckets.length;
  }

  private totalBytesUsed() {
    // ts-ignore
    return prettyBytes(this.state.used_bytes_total);
  }

  render() {
    return (
      <Container>
        <Row>
          <Col>
            <Card className="m-1">
              <Card.Body>
                <Card.Title>Status</Card.Title>
                <Card.Text className="h5">
                  <Badge variant={this.statusAsVariant()}>{this.statusAsText()}</Badge>
                </Card.Text>
              </Card.Body>
            </Card>
          </Col>
          <Col>
            <Card className="m-1">
              <Card.Body>
                <Card.Title>Data Bucket Canisters</Card.Title>
                <Card.Text className="h5">
                  <Badge variant="secondary">{this.numDataBuckets()}</Badge>
                </Card.Text>
              </Card.Body>
            </Card>
          </Col>
          <Col>
            <Card className="m-1">
              <Card.Body>
                <Card.Title>Total Data Stored</Card.Title>
                <Card.Text className="h5">
                  <Badge variant="primary">{this.totalBytesUsed()}</Badge>
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
