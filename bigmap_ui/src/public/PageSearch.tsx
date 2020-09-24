
import React from 'react';
import { Container, Table, Form, FormControl, Button } from 'react-bootstrap';
import { bigMapSearch } from '../utils';

interface Search {
  query: string;
  results: SearchResults | null
}

class PageOverview extends React.Component<{}, Search> {
  constructor(props: any) {
    super(props);
    this.state = {
      query: '',
      results: null
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

  private renderSearchResults() {
    if (this.state.results) {
      return this.state.results.entries.map((item, index) => <tr key={index}><td>{item.key}</td><td>{item.value}</td></tr>);
    } else {
      return <tr key="0"></tr>;
    }
  }

  private renderSearchHits() {
    if (this.state.results) {
      if (this.state.results.entries_count < 20) {
        return <p><br />{this.state.results.entries_count} hits</p>;
      } else {
        return <p><br />About {this.state.results.entries_count} hits</p>;
      }
    } else {
      return <br />;
    }
  }

  render() {
    return (
      <Container>
        <Form inline onSubmit={this.handleSubmit} action="#">
          <FormControl type="text" placeholder="Search" style={{ width: '80%' }} onChange={this.handleQueryChange} />
          <Button variant="info" type="submit">Search</Button>
        </Form>

        {this.renderSearchHits()}

        <Table striped bordered hover>
          <tbody>
            {this.renderSearchResults()}
          </tbody>
        </Table>
      </Container>
    );
  }
}

export default PageOverview;
