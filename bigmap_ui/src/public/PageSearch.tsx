
import React from 'react';
import { Container, Table, Form, FormControl, Button } from 'react-bootstrap';
import { bigMapSearch } from '../utils';

interface Search {
  query: string;
  results: SearchResultItem[]
}

class PageOverview extends React.Component<{}, Search> {
  constructor(props: any) {
    super(props);
    this.state = {
      query: '',
      results: []
    }
    this.handleQueryChange = this.handleQueryChange.bind(this);
    this.handleSubmit = this.handleSubmit.bind(this);
  }

  private handleQueryChange(event: React.ChangeEvent<HTMLInputElement>) {
    let q: string = event.target.value;
    this.setState({ query: q });
  }

  private async handleSubmit(event: any) {
    event.preventDefault();
    event.stopPropagation();
    console.log("Start query: " + this.state.query);
    this.setState({ results: await bigMapSearch(this.state.query) });
  };

  render() {
    return (
      <Container>
        <Form inline onSubmit={this.handleSubmit} action="#">
          <FormControl type="text" placeholder="Search" style={{ width: '80%' }} onChange={this.handleQueryChange} />
          <Button variant="info" type="submit">Search</Button>
        </Form>

        <h3 className="header p-3">Search results</h3><br />

        <Table striped bordered hover>
          <tbody>
            {
              this.state.results.map((item, index) => <tr key={index}><td>{item.key}</td><td>{item.value}</td></tr>)
            }
          </tbody>
        </Table>
      </Container>
    );
  }
}

export default PageOverview;
