import "./Highlight.css";
import Star from "@mui/icons-material/Star"
import StarBorder from "@mui/icons-material/StarBorder"
import Warning from "@mui/icons-material/Warning"

function Highlight({label, category}) {
  const categoryIcons = {
    great: <Star className="highlight-icon highlight-great"/>,
    good: <StarBorder className="highlight-icon highlight-good"/>,
    bad: <Warning className="highlight-icon highlight-bad"/>,
    worse: <Warning className="highlight-icon highlight-worse"/>,
  };

  console.log(category);
  return (
    <span>{categoryIcons[category]}{label}</span>
  );
}

export default Highlight;
