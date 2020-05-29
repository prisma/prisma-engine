package queries.orderAndPagination

import org.scalatest.{FlatSpec, Matchers}
import util.ConnectorCapability.JoinRelationLinksCapability
import util._

class NonEmbeddedOrderBySpec extends FlatSpec with Matchers with ApiSpecBase {
  override def runOnlyForCapabilities = Set(JoinRelationLinksCapability)

  val testDataModels = {
    val s1 = """
      model  List {
        id    String @id @default(cuid())
        name  String @unique

        todos Todo[]
      }

      model  Todo {
        id    String @id @default(cuid())
        title String @unique

        lists List[]
      }
    """

    val s2 = """
      model  List {
        id    String @id @default(cuid())
        name  String @unique

        todos Todo[]
      }

      model  Todo {
        id    String @id @default(cuid())
        title String @unique

        lists List[]
      }
    """

    TestDataModels(mongo = Vector(s1), sql = Vector(s2))
  }

//  this is not the intended behaviour anymore since we remove implicit ordering by id
  "The order when not using order by" should "be the same no matter if pagination is used or not" ignore {
    testDataModels.testV11 { project =>
      createLists(project)
      createTodos(project)

      val result1 = server.query(
        """
        |{
        |  list(where: {name: "1"}) {
        |    name
        |    todos{
        |      title
        |    }
        |  }
        |}
      """,
        project
      )

      val result2 = server.query(
        """
        |{
        |  list(where: {name: "1"}) {
        |    name
        |    todos(take:10){
        |      title
        |    }
        |  }
        |}
      """,
        project
      )

      result1 should be(result2)

      val result3 = server.query(
        """
        |{
        |  todo(where: {title: "1"}) {
        |    title
        |    lists{
        |      name
        |    }
        |  }
        |}
      """,
        project
      )

      val result4 = server.query(
        """
        |{
        |  todo(where: {title: "1"}) {
        |    title
        |    lists(take:10){
        |      name
        |    }
        |  }
        |}
      """,
        project
      )

      result3 should be(result4)
    }
  }

  private def createLists(project: Project): Unit = {
    server.query(
      """
        |mutation {
        |  a: createList(data: {name: "1"}){ id }
        |  b: createList(data: {name: "2"}){ id }
        |  d: createList(data: {name: "4"}){ id }
        |  f: createList(data: {name: "6"}){ id }
        |  g: createList(data: {name: "7"}){ id }
        |  c: createList(data: {name: "3"}){ id }
        |  e: createList(data: {name: "5"}){ id }
        |}
      """,
      project
    )
  }

  private def createTodos(project: Project): Unit = {
    server.query(
      """
        |mutation {
        |  a: createTodo(data: {title: "1", lists: { connect: [{name:"1"},{name:"2"},{name:"3"}]}}){ id }
        |  c: createTodo(data: {title: "3", lists: { connect: [{name:"1"},{name:"2"},{name:"3"}]}}){ id }
        |  d: createTodo(data: {title: "4", lists: { connect: [{name:"1"},{name:"2"},{name:"3"}]}}){ id }
        |  f: createTodo(data: {title: "6", lists: { connect: [{name:"1"},{name:"2"},{name:"3"}]}}){ id }
        |  g: createTodo(data: {title: "7", lists: { connect: [{name:"1"},{name:"2"},{name:"3"}]}}){ id }
        |  b: createTodo(data: {title: "2", lists: { connect: [{name:"1"},{name:"2"},{name:"3"}]}}){ id }
        |  e: createTodo(data: {title: "5", lists: { connect: [{name:"1"},{name:"2"},{name:"3"}]}}){ id }
        |}
      """,
      project
    )
  }
}
