import { Link } from "react-router-dom"
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  navigationMenuTriggerStyle,
} from "@/components/ui/navigation-menu"

export function NavigationBar() {
  return (
    <NavigationMenu>
      <NavigationMenuList>
        
        {/* Button 1: General DAQ */}
        <NavigationMenuItem>
          <NavigationMenuLink asChild className={navigationMenuTriggerStyle()}>
            <Link to="/">Home</Link>
          </NavigationMenuLink>
        </NavigationMenuItem>
        <NavigationMenuItem>
          <NavigationMenuLink asChild className={navigationMenuTriggerStyle()}>
            <Link to="/daq">DAQ Monitor</Link>
          </NavigationMenuLink>
        </NavigationMenuItem>

        {/* Button 2: BMS Monitoring */}
        <NavigationMenuItem>
          <NavigationMenuLink asChild className={navigationMenuTriggerStyle()}>
            <Link to="/bms">BMS Monitor</Link>
          </NavigationMenuLink>
        </NavigationMenuItem>

        {/* Button 3: Logs */}
        <NavigationMenuItem>
          <NavigationMenuLink asChild className={navigationMenuTriggerStyle()}>
            <Link to="/logs">CAN Logs</Link>
          </NavigationMenuLink>
        </NavigationMenuItem>

      </NavigationMenuList>
    </NavigationMenu>
  )
}