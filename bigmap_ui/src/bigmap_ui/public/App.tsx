import React from 'react';
import ReactDOM from 'react-dom';

import tsreact from 'ic:canisters/tsreact_v2';
import {test} from './test';

interface AppState {
  count: number;
}
interface AppProps {}

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
      <section>
        <h2>Count: {count}</h2>
        <h2>
          <button onClick={this.increment}>Click to Add</button>
          <button onClick={this.decrement}>Click to Subtract</button>
        </h2>
      </section>
    );
  }
}

/*
NB: dfx bootstrap's index.html generated looks like this:

<app id="app"><progress class="ic_progress" id="ic-progress">Loading...</progress></app>
*/
ReactDOM.render(<App />, document.getElementById('app'));
