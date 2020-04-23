import React, { useState, useEffect } from 'react';
import './App.css';
import graphql from 'babel-plugin-relay/macro';
import { QueryRenderer } from 'react-relay';
import environment from './environment';

function redirect(url) {
  window.open(url, '_blank');
}

function NotebookView(props) {
  const {currentPage, setCurrentPage, notebook} = props;
  useEffect(() => {
    if (currentPage != null) {
      function keyListener(e) {
        if (e.key === 'ArrowLeft') {
          if (currentPage > 0) {
            setCurrentPage(currentPage - 1);
          }
        }
        if (e.key == 'ArrowRight') {
          if (currentPage < notebook.pages.length - 1) {
            setCurrentPage(currentPage + 1);
          }
        }      
      }
      document.addEventListener('keydown', keyListener);
      return () => {
        document.removeEventListener('keydown', keyListener);
      };
    }
  }, [currentPage, setCurrentPage]);

  return (
    <div className={"App-notebooks full"}>
      <ul className="App-pages">
        {notebook.pages
          .map((page, i) => {
            if (currentPage !== null && i !== currentPage) {
              return null;
            }
            return (
              <li className="page svg" key={i} onClick={_ => {
                setCurrentPage(null);
              }}>
                <div className="page-flex">
                  <img src={notebook.pages[i].svg + `#` + Math.random()} key={Math.random()} alt={`Page ${i + 1}`} className="page-flex-image" />
                  <div className="page-flex-info">
                    <h2>Page {i + 1}<br /></h2>
                    <p>Modified in {notebook.pages[i].modified}</p>
                    <div className="page-flex-info-download">
                      Download:
                      <button onClick={() => redirect(notebook.pages[i].svg)}>SVG</button>
                      <button onClick={() => redirect(notebook.pages[i].svg)}>PDF</button>
                      <button onClick={e => {
                        redirect('http://localhost:8080' + notebook.pages[i].png);
                        e.preventDefault();
                        e.stopPropagation();
                      }}>PNG</button>
                    </div>
                  </div>
                </div>
              </li>);
        })}
      </ul>
    </div>
  );
}

function NotebookIcons(props) {
  const {notebook, setCurrentPage} = props;
  return (
    <div className={"App-notebooks icons"}>
      <ul className="App-pages">
        {notebook.pages
          .map((page, i) => {
            const src = notebook.pages[i].svg;
            return (
              <li className="page svg" key={i} onClick={_ => {
                setCurrentPage(i);
              }}>
                {src == null ? `Page ${i + 1}` : <img src={`${src}#${Math.random()}`} alt={`Page ${i + 1}`} /> }
              </li>);
        })}
      </ul>
    </div>
  );
}

function Sidebar(props) {
  const {notebooks, currentNotebook, setCurrentNotebook, setCurrentPage} = props;
  return (
    <ul className="App-sidebar">
      <h2 style={{textAlign: 'center'}}>checkMark</h2>
      {notebooks
        .map((notebook, i) => {
          return (
            <li
              key={i}
              onClick={_ => {
                setCurrentNotebook(i);
                setCurrentPage(null);
              }}
              className={currentNotebook === i ? "selected" : ""}
            >{notebook.name} <span className="size">{notebook.pages.length}</span></li>
          );
        })}
    </ul>
  );
}

function App() {
  const [currentNotebook, setCurrentNotebook] = useState(0);
  const [currentPage, setCurrentPage] = useState(null);

  return (
    <QueryRenderer
      environment={environment}
      query={graphql`
        query AppQuery {
          notebooks {
            id
            name
            pages {
              id
              svg
              png
              modified
            }
          }
        }
      `}
      variables={{
      }}
      render={({error, props}) => {
        if (!props || !props.notebooks) {
          return <div>Loading...</div> 
        }

        const notebooks = props.notebooks.slice().sort((a, b) => {
          return a.name < b.name ? -1 : a.name > b.name ? 1 : 0;
        });

        const notebook = notebooks[currentNotebook];

        return (
          <div className="App">
            <Sidebar
              {...{notebooks, currentNotebook, setCurrentPage, setCurrentNotebook}}
            />
            {!notebook ? null : <NotebookIcons
              {...{notebook, setCurrentPage}}
              />}
            {currentPage == null ? null :
              <NotebookView
                {...{notebook, setCurrentPage, currentPage}}
              />}
          </div>
        );
      }}
    />
  );
}

export default App;
