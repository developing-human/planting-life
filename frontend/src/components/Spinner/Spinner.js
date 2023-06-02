import "./Spinner.css";

function Spinner() {
  return (
    <div className="spinner">
      <img src={`${process.env.PUBLIC_URL}/loading-earth.png`} alt="Loading" />
    </div>
  );
}

export default Spinner;
