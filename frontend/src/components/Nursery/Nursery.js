import Card from "@mui/material/Card";
import CardHeader from "@mui/material/CardHeader";
import CardContent from "@mui/material/CardContent";
import Typography from "@mui/material/Typography";
import { styled } from '@mui/system'

// styling
//import "./Nursery.css"

function Nursery({ nursery }) {
  const NoPaddingCardContent = styled(CardContent)({
    paddingTop: '0px',
  });

  return (

    <Card sx={{ width: 350, maxWidth: "90vw", minHeight: 125, maxHeight: 575 }}>
      <CardHeader title={nursery.name} subheader={`${nursery.miles} miles away`}/>

      <NoPaddingCardContent>
        <Typography variant="body2" color="text.secondary">
          {nursery.address}, {nursery.city}, {nursery.state} {nursery.zip} <br />
          <a href={nursery.url} target="blank">Website</a> 
          &nbsp;|&nbsp; 
          <a href={nursery.map_url} target="blank">Map</a>
        </Typography>
      </NoPaddingCardContent>
    </Card>
  );
}

export default Nursery;
