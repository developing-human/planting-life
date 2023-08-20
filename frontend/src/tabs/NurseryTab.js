import Nursery from "../components/Nursery";

const NurseryTab = ({ nurseries }) => {
  return (
    <section className="card-container">
      {nurseries.map((nursery, index) => (
        <Nursery nursery={nursery} key={index} />
      ))}
    </section>
  );
};

export default NurseryTab;
